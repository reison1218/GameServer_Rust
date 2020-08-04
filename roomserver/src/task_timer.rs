use crate::mgr::room_mgr::RoomMgr;
use crate::room::member::MemberState;
use crate::room::room::{MemberLeaveNoticeType, RoomState, MEMBER_MAX};
use crate::room::room_model::RoomModel;
use chrono::Local;
use log::{error, info, warn};
use serde_json::Value as JsonValue;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub enum TaskCmd {
    MatchRoomStart = 101,  //匹配房间开始任务
    ChoiceIndex = 102,     //选择占位
    ChoiceTurnOrder = 103, //选择回合顺序
    BattleTurnTime = 104,  //战斗时间回合限制
}

impl From<u16> for TaskCmd {
    fn from(v: u16) -> Self {
        if v == TaskCmd::MatchRoomStart as u16 {
            return TaskCmd::MatchRoomStart;
        }
        if v == TaskCmd::ChoiceIndex as u16 {
            return TaskCmd::ChoiceIndex;
        }
        if v == TaskCmd::ChoiceTurnOrder as u16 {
            return TaskCmd::ChoiceTurnOrder;
        }
        if v == TaskCmd::BattleTurnTime as u16 {
            return TaskCmd::BattleTurnTime;
        }
        TaskCmd::MatchRoomStart
    }
}

#[derive(Debug, Clone, Default)]
pub struct Task {
    pub cmd: u16,        //要执行的命令
    pub delay: u64,      //要延迟执行的时间
    pub data: JsonValue, //数据
}

///初始化定时执行任务
pub fn init_timer(rm: Arc<RwLock<RoomMgr>>) {
    let m = move || {
        let (sender, rec) = crossbeam::crossbeam_channel::bounded(1024);
        let mut write = rm.write().unwrap();
        write.task_sender = Some(sender);
        std::mem::drop(write);

        loop {
            let res = rec.recv();
            if res.is_err() {
                error!("{:?}", res.err().unwrap());
                continue;
            }
            let task = res.unwrap();
            let task_cmd = TaskCmd::from(task.cmd);
            let rm_clone = rm.clone();
            match task_cmd {
                TaskCmd::MatchRoomStart => {
                    let m = || {
                        std::thread::sleep(Duration::from_millis(task.delay));
                        match_room_start(rm_clone, task);
                    };
                    //设置线程名字和堆栈大小
                    let thread_builder = std::thread::Builder::new()
                        .name("TIMER_THREAD_TASK".to_owned())
                        .stack_size(128 * 1024);
                    let res = thread_builder.spawn(m);
                    if res.is_err() {
                        error!("{:?}", res.err().unwrap());
                    }
                }
                TaskCmd::ChoiceIndex => {
                    let m = || {
                        std::thread::sleep(Duration::from_millis(task.delay));
                        choice_index(rm_clone, task);
                    };
                    //设置线程名字和堆栈大小
                    let thread_builder = std::thread::Builder::new()
                        .name("TIMER_THREAD_TASK".to_owned())
                        .stack_size(128 * 1024);
                    let res = thread_builder.spawn(m);
                    if res.is_err() {
                        error!("{:?}", res.err().unwrap());
                    }
                }
                TaskCmd::ChoiceTurnOrder => {
                    let m = || {
                        std::thread::sleep(Duration::from_millis(task.delay));
                        choice_turn(rm_clone, task);
                    };
                    //设置线程名字和堆栈大小
                    let thread_builder = std::thread::Builder::new()
                        .name("TIMER_THREAD_TASK".to_owned())
                        .stack_size(128 * 1024);
                    let res = thread_builder.spawn(m);
                    if res.is_err() {
                        error!("{:?}", res.err().unwrap());
                    }
                }
                TaskCmd::BattleTurnTime => {
                    let m = || {
                        std::thread::sleep(Duration::from_millis(task.delay));
                        battle_turn_time(rm_clone, task);
                    };
                    //设置线程名字和堆栈大小
                    let thread_builder = std::thread::Builder::new()
                        .name("TIMER_THREAD_TASK".to_owned())
                        .stack_size(128 * 1024);
                    let res = thread_builder.spawn(m);
                    if res.is_err() {
                        error!("{:?}", res.err().unwrap());
                    }
                }
            }
        }
    };
    let timer_thread = std::thread::Builder::new().name("TIMER_THREAD".to_owned());
    let res = timer_thread.spawn(m);
    if res.is_err() {
        error!("{:?}", res.err().unwrap());
        std::process::abort();
    }
    info!("初始化定时器任务执行器成功!");
}

///执行匹配房间任务
fn match_room_start(rm: Arc<RwLock<RoomMgr>>, task: Task) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();
    let battle_type = map.get("battle_type");
    if battle_type.is_none() {
        return;
    }
    let battle_type = battle_type.unwrap().as_u64();
    if battle_type.is_none() {
        return;
    }
    let battle_type = battle_type.unwrap() as u8;

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

    let mut write = rm.write().unwrap();

    let match_room = write.match_rooms.get_match_room_mut(&battle_type);

    let room = match_room.get_room_mut(&room_id);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();
    //如果房间已经不再等待阶段了，就什么都不执行
    if room.get_state() != &RoomState::Await {
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
    if room.get_member_count() as u8 != MEMBER_MAX {
        let mut v = Vec::new();
        for member in room.members.values() {
            if member.state == MemberState::Ready as u8 {
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
            if member.state == MemberState::NotReady as u8 {
                v.push(member.user_id);
            }
        }
        if v.len() > 0 {
            let mut rm_v = Vec::new();
            for member_id in &v[..] {
                let res =
                    match_room.leave_room(MemberLeaveNoticeType::Kicked as u8, &room_id, member_id);
                if res.is_err() {}
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
                write.player_room.remove(&member_id);
            }
            return;
        }
    }
    //执行开始逻辑
    room.start();
}

///占位任务，没选的直接t出房间
fn choice_index(rm: Arc<RwLock<RoomMgr>>, task: Task) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();
    let user_id = map.get("user_id");
    if user_id.is_none() {
        return;
    }
    let user_id = user_id.unwrap().as_u64();
    if user_id.is_none() {
        return;
    }
    let user_id = user_id.unwrap() as u32;

    let mut write = rm.write().unwrap();

    let room = write.get_room_mut(&user_id);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();

    //判断房间状态
    if room.get_state() != &RoomState::ChoiceIndex {
        return;
    }
    let next_user = room.get_turn_user(None);
    if let Err(_) = next_user {
        return;
    }
    let next_user = next_user.unwrap();
    //判断是否轮到自己
    if next_user != user_id {
        return;
    }
    info!("定时检测选占位任务,没有选择都人T出去,user_id:{}", user_id);
    room.remove_member(MemberLeaveNoticeType::Kicked as u8, &user_id);
    write.player_room.remove(&user_id);
}

///选择占位,超时了就跳过，如果是最后一个人超时，则系统帮忙给未选择的人随机分配
fn choice_turn(rm: Arc<RwLock<RoomMgr>>, task: Task) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();
    let user_id = map.get("user_id");
    if user_id.is_none() {
        return;
    }
    let user_id = user_id.unwrap().as_u64();
    if user_id.is_none() {
        return;
    }
    let user_id = user_id.unwrap() as u32;

    let mut write = rm.write().unwrap();

    let room = write.get_room_mut(&user_id);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();

    //判断房间状态
    if room.get_state() != &RoomState::ChoiceTurn {
        return;
    }

    //判断当前是不是轮到自己选
    let next_user = room.get_choice_user(None);
    if let Err(e) = next_user {
        warn!("{:?}", e);
        return;
    }
    let next_user = next_user.unwrap();
    if next_user != user_id {
        warn!(
            "timer choice_turn next_user!=user_id!next_user:{},user_id:{}",
            next_user, user_id
        );
        return;
    }

    //跳过
    room.skip_choice_turn(user_id);
}

fn battle_turn_time(rm: Arc<RwLock<RoomMgr>>, task: Task) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();
    let user_id = map.get("user_id");
    if user_id.is_none() {
        return;
    }
    let user_id = user_id.unwrap().as_u64();
    if user_id.is_none() {
        return;
    }
    let user_id = user_id.unwrap() as u32;

    let mut write = rm.write().unwrap();

    let room = write.get_room_mut(&user_id);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();

    let next_user_id = room.get_turn_user(None);
    if let Err(e) = next_user_id {
        warn!("{:?}", e);
        return;
    }
    let next_user_id = next_user_id.unwrap();
    if next_user_id != user_id {
        return;
    }
    //如果玩家啥都没做，就T出房间
    if room.is_battle_do_nothing() {
        room.remove_member(MemberLeaveNoticeType::Kicked as u8, &user_id);
    }
}
