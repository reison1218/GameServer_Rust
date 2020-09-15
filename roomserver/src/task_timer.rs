use crate::mgr::room_mgr::RoomMgr;
use crate::room::member::MemberState;
use crate::room::room::{MemberLeaveNoticeType, RoomState, MEMBER_MAX};
use crate::room::room_model::{BattleType, MatchRoom, RoomModel};
use crate::SCHEDULED_MGR;
use chrono::Local;
use log::{error, info, warn};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde_json::Value as JsonValue;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum TaskCmd {
    MatchRoomStart = 101,     //匹配房间开始任务
    ChoiceIndex = 102,        //选择占位
    ChoiceTurnOrder = 103,    //选择回合顺序
    BattleTurnTime = 104,     //战斗时间回合限制
    MaxBattleTurnTimes = 105, //战斗turn达到最大
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
pub fn init_timer(rm: Arc<Mutex<RoomMgr>>) {
    let m = move || {
        let (sender, rec) = crossbeam::crossbeam_channel::bounded(1024);
        let mut lock = rm.lock().unwrap();
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
                TaskCmd::MatchRoomStart => match_room_start,
                TaskCmd::ChoiceIndex => choice_index,
                TaskCmd::ChoiceTurnOrder => choice_turn,
                TaskCmd::BattleTurnTime => battle_turn_time,
                TaskCmd::MaxBattleTurnTimes => max_battle_turn_limit,
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
fn match_room_start(rm: Arc<Mutex<RoomMgr>>, task: Task) {
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
    let battle_type = BattleType::try_from(battle_type.unwrap() as u8).unwrap();

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

    let mut lock = rm.lock().unwrap();

    let match_room = lock.match_rooms.get_match_room_mut(battle_type);
    let match_room_ptr = match_room as *mut MatchRoom;
    let room = match_room.get_room_mut(&room_id);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();
    //如果房间已经不再等待阶段了，就什么都不执行
    if room.get_state() != RoomState::Await {
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
                lock.player_room.remove(&member_id);
            }
            unsafe {
                let room = match_room_ptr.as_mut().unwrap().get_room_mut(&room_id);
                if let Some(room) = room {
                    if !room.is_empty() {
                        return;
                    }
                    let room_type = room.get_room_type();
                    let battle_type = room.setting.battle_type;
                    let room_id = room.get_room_id();
                    let v = room.get_member_vec();
                    lock.rm_room(room_id, room_type, battle_type, v);
                }
            }
            return;
        }
    }
    //执行开始逻辑
    room.start();
}

///占位任务，没选的直接t出房间
fn choice_index(rm: Arc<Mutex<RoomMgr>>, task: Task) {
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

    let mut lock = rm.lock().unwrap();

    let room = lock.get_room_mut(&user_id);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();

    //判断房间状态
    if room.state != RoomState::ChoiceIndex {
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
    let need_rm_room;
    room.remove_member(MemberLeaveNoticeType::Kicked.into(), &user_id);
    if room.state == RoomState::BattleOvered {
        need_rm_room = true
    } else {
        need_rm_room = false;
    }
    if need_rm_room {
        let room_type = room.get_room_type();
        let battle_type = room.setting.battle_type;
        let room_id = room.get_room_id();
        let v = room.get_member_vec();
        lock.rm_room(room_id, room_type, battle_type, v);
    }
    lock.player_room.remove(&user_id);
}

///选择占位,超时了就跳过，如果是最后一个人超时，则系统帮忙给未选择的人随机分配
fn choice_turn(rm: Arc<Mutex<RoomMgr>>, task: Task) {
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

    let mut lock = rm.lock().unwrap();

    let room = lock.get_room_mut(&user_id);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();

    //判断房间状态
    if room.state != RoomState::ChoiceTurn {
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

fn battle_turn_time(rm: Arc<Mutex<RoomMgr>>, task: Task) {
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

    let mut lock = rm.lock().unwrap();

    let room = lock.get_room_mut(&user_id);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();

    //校验房间状态
    if room.state != RoomState::BattleStarted {
        warn!(
            "battle_turn_time,the room state is not RoomState::BattleStarted!room_id:{}",
            room.get_room_id()
        );
        return;
    }

    //校验当前是不是这个人
    let next_user_id = room.get_turn_user(None);
    if let Err(e) = next_user_id {
        warn!("{:?}", e);
        return;
    }
    let next_user_id = next_user_id.unwrap();
    if next_user_id != user_id {
        return;
    }

    let battle_cter = room.battle_data.get_battle_cter(Some(user_id), true);
    if let Err(e) = battle_cter {
        warn!("{:?}", e);
        return;
    }
    let battle_cter = battle_cter.unwrap();

    //如果玩家啥都没做，就T出房间
    if battle_cter.open_cell_vec.is_empty() {
        room.remove_member(MemberLeaveNoticeType::Kicked as u8, &user_id);
    }
    let is_empty = room.is_empty();
    if is_empty {
        let room_type = room.get_room_type();
        let battle_type = room.setting.battle_type;
        let room_id = room.get_room_id();
        let v = room.get_member_vec();
        lock.rm_room(room_id, room_type, battle_type, v);
    }
}

pub fn max_battle_turn_limit(rm: Arc<Mutex<RoomMgr>>, task: Task) {
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

    let mut lock = rm.lock().unwrap();

    let room = lock.get_room_mut(&user_id);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();
    let room_type = room.get_room_type();
    let battle_type = room.setting.battle_type;
    let room_id = room.get_room_id();
    let v = room.get_member_vec();

    lock.rm_room(room_id, room_type, battle_type, v);
}
