use crate::room::match_room::MatchRoom;
use crate::room::member::MemberState;
use crate::room::room::{MemberLeaveNoticeType, RoomState, MEMBER_MAX};
use crate::room::room_model::{RoomModel, RoomType};
use crate::{Lock, SCHEDULED_MGR};
use async_std::task::block_on;
use chrono::Local;
use crossbeam::channel::Sender;
use log::{error, info, warn};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use protobuf::Message;
use serde_json::{Map, Value as JsonValue};
use std::borrow::BorrowMut;
use std::convert::TryFrom;
use std::str::FromStr;
use std::time::Duration;
use tools::cmd_code::ClientCode;
use tools::protos::room::S_INTO_ROOM_CANCEL_NOTICE;

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum TaskCmd {
    MatchRoomReady = 101,       //匹配房间准备
    MatchRoomConfirmInto = 102, //匹配房间，进入房间
}

impl TaskCmd {
    pub fn from(value: u16) -> Self {
        TaskCmd::try_from(value).unwrap()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Task {
    pub cmd: u16,        //要执行的命令
    pub delay: u64,      //要延迟执行的时间
    pub data: JsonValue, //数据
}

///初始化定时执行任务
pub fn init_timer(rm: Lock) {
    let m = move || {
        let (sender, rec) = crossbeam::channel::bounded(1024);
        let mut lock = block_on(rm.lock());
        lock.task_sender = Some(sender);
        std::mem::drop(lock);

        loop {
            let res = rec.recv();
            if let Err(e) = res {
                error!("{:?}", e);
                continue;
            }
            let task = res.unwrap();
            let delay = task.delay;

            let task_cmd = TaskCmd::from(task.cmd);
            let rm_clone = rm.clone();
            let f = match task_cmd {
                TaskCmd::MatchRoomReady => match_room_ready,
                TaskCmd::MatchRoomConfirmInto => match_room_confirm_into,
            };
            let m = move || f(rm_clone, task);
            SCHEDULED_MGR.execute_after(Duration::from_millis(delay), m);
        }
    };
    let timer_thread = std::thread::Builder::new().name("TIMER_THREAD".to_owned());
    let res = timer_thread.spawn(m);
    if let Err(e) = res {
        error!("{:?}", e);
        std::process::abort();
    }
    info!("初始化定时器任务执行器成功!");
}

///执行匹配房间任务
fn match_room_confirm_into(rm: Lock, task: Task) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();

    let room_id = map.get("room_id");
    if room_id.is_none() {
        return;
    }
    let room_id = room_id.unwrap();
    let room_id = room_id.as_u64();
    if room_id.is_none() {
        return;
    }
    let room_id = room_id.unwrap() as u32;

    let room_type = map.get("room_type");
    if room_type.is_none() {
        return;
    }
    let room_type = room_type.unwrap();
    let room_type = room_type.as_u64();
    if room_type.is_none() {
        return;
    }
    let room_type = room_type.unwrap() as u8;
    let room_type = RoomType::try_from(room_type).unwrap();

    let mut lock = block_on(rm.lock());
    let room;
    match room_type {
        RoomType::OneVOneVOneVOneMatch => {
            let match_room = lock.match_room.borrow_mut();
            let res = match_room.get_room_mut(&room_id);
            if res.is_none() {
                return;
            }
            room = res.unwrap();
        }
        RoomType::WorldBoseMatch => {
            let match_room = lock.world_boss_match_room.borrow_mut();
            let res = match_room.get_room_mut(&room_id);
            if res.is_none() {
                return;
            }
            room = res.unwrap();
        }
        _ => todo!(),
    }

    //如果房间已经不再等待阶段了，就什么都不执行
    if room.get_state() != RoomState::AwaitConfirm {
        return;
    }
    let mut need_rm = false;
    //判断是否需要删除房间
    for member in room.members.values() {
        if member.state == MemberState::AwaitConfirm {
            need_rm = true;
            break;
        }
    }
    if !need_rm {
        return;
    }
    //解散房间，并通知所有客户端
    let sircn = S_INTO_ROOM_CANCEL_NOTICE::new();
    let bytes = sircn.write_to_bytes().unwrap();
    room.send_2_all_client(ClientCode::IntoRoomCancelNotice, bytes);

    //删除房间
    lock.rm_room_without_push(room_type, room_id);
}

///执行匹配房间任务
fn match_room_ready(rm: Lock, task: Task) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();

    let room_id = map.get("room_id");
    if room_id.is_none() {
        return;
    }
    let room_id = room_id.unwrap();
    let room_id = room_id.as_u64();
    if room_id.is_none() {
        return;
    }
    let room_id = room_id.unwrap() as u32;

    let mut lock = block_on(rm.lock());

    let match_room = lock.match_room.borrow_mut();
    let match_room_ptr = match_room as *mut MatchRoom;
    let room = match_room.get_room_mut(&room_id);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();
    //如果房间已经不再等待阶段了，就什么都不执行
    if room.get_state() != RoomState::AwaitReady {
        return;
    }

    let now_time = Local::now();
    let now_time = now_time.timestamp_millis() as u64;
    //判断是否有新人进来了
    for (_, member) in room.members.iter() {
        if now_time - member.join_time < task.delay {
            info!("有新成员进来，定时检测房间开始任务取消");
            return;
        }
    }

    //如果人未满，则取消准备
    if room.get_member_count() != MEMBER_MAX {
        let mut v = Vec::new();
        for member in room.members.values() {
            if member.state == MemberState::Ready {
                v.push(member.user_id);
            }
        }
        for member_id in v {
            info!(
                "定时检测房间开始任务,取消其他成员准备,user_id:{}",
                member_id
            );
            room.prepare_cancel(&member_id, false);
        }
        return;
    } else {
        //满都就把未准备都玩家t出去
        let mut v = Vec::new();
        for member in room.members.values() {
            if member.state == MemberState::NotReady {
                v.push(member.user_id);
            }
        }
        if v.len() > 0 {
            let mut rm_v = Vec::new();
            for member_id in &v[..] {
                let res = match_room.leave_room(
                    MemberLeaveNoticeType::Kicked as u8,
                    &room_id,
                    member_id,
                    true,
                    true,
                );
                match res {
                    Ok(_) => {
                        rm_v.push(*member_id);
                        info!(
                            "由于匹配房人满，倒计时有玩家未准备，将未准备的玩家T出房间！room_id:{},user_id:{}",
                            room_id, member_id
                        );
                    }
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
            }

            for member_id in rm_v {
                lock.player_room.remove(&member_id);
            }
            unsafe {
                let room = match_room_ptr.as_mut().unwrap().get_room_mut(&room_id);
                if let Some(room) = room {
                    if !room.is_empty() {
                        return;
                    }
                    let room_type = room.get_room_type();
                    let room_id = room.get_room_id();
                    lock.rm_room_without_push(room_type, room_id);
                }
            }
            return;
        }
    }
    //执行开始逻辑
    room.start();
}

pub fn build_match_room_ready_task(room_id: u32, task_sender: Sender<Task>) {
    //创建延迟任务，并发送给定时器接收方执行
    let mut task = Task::default();
    let time_limit = crate::TEMPLATES
        .constant_temp_mgr()
        .temps
        .get("kick_not_prepare_time");
    if let Some(time) = time_limit {
        let time = u64::from_str(time.value.as_str());
        match time {
            Ok(time) => task.delay = time + 500,
            Err(e) => {
                error!("{:?}", e)
            }
        }
    } else {
        task.delay = 60000_u64;
        warn!("the Constant kick_not_prepare_time is None!pls check!");
    }

    task.cmd = TaskCmd::MatchRoomReady as u16;
    let mut map = Map::new();
    map.insert("room_id".to_owned(), JsonValue::from(room_id));
    task.data = JsonValue::from(map);
    let res = task_sender.send(task);
    if let Err(e) = res {
        error!("{:?}", e);
    }
}

pub fn build_confirm_into_room_task(room_type: RoomType, room_id: u32, task_sender: Sender<Task>) {
    //创建延迟任务，并发送给定时器接收方执行
    let mut task = Task::default();
    let time_limit = crate::TEMPLATES
        .constant_temp_mgr()
        .temps
        .get("match_room_be_sure_limit_time");
    if let Some(time) = time_limit {
        let time = u64::from_str(time.value.as_str());
        match time {
            Ok(time) => task.delay = time + 500,
            Err(e) => {
                error!("{:?}", e)
            }
        }
    } else {
        task.delay = 60000_u64;
        warn!("the Constant match_room_be_sure_limit_time is None!pls check!");
    }

    task.cmd = TaskCmd::MatchRoomConfirmInto as u16;
    let mut map = Map::new();
    map.insert("room_id".to_owned(), JsonValue::from(room_id));
    map.insert("room_type".to_owned(), JsonValue::from(room_type.into_u8()));
    task.data = JsonValue::from(map);
    let res = task_sender.send(task);
    if let Err(e) = res {
        error!("{:?}", e);
    }
}
