use crate::entity::member::MemberState;
use crate::entity::room::MemberLeaveNoticeType;
use crate::entity::room_model::RoomModel;
use crate::mgr::room_mgr::RoomMgr;
use log::{error, info};
use serde_json::Value as JsonValue;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub enum TaskCmd {
    MatchRoomStart = 101,  //匹配房间开始任务
    ChoiceLocation = 102,  //选择占位
    ChoiceTurnOrder = 103, //选择回合顺序
    BattleTurnTime = 104,  //战斗时间回合限制
}

impl From<u16> for TaskCmd {
    fn from(v: u16) -> Self {
        if v == TaskCmd::MatchRoomStart as u16 {
            return TaskCmd::MatchRoomStart;
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
                        .stack_size(32 * 1024);
                    let res = thread_builder.spawn(m);
                    if res.is_err() {
                        error!("{:?}", res.err().unwrap());
                    }
                }
                TaskCmd::ChoiceLocation => {
                    let m = || {
                        std::thread::sleep(Duration::from_millis(task.delay));
                        choice_location(rm_clone, task);
                    };
                    //设置线程名字和堆栈大小
                    let thread_builder = std::thread::Builder::new()
                        .name("TIMER_THREAD_TASK".to_owned())
                        .stack_size(32 * 1024);
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
                        .stack_size(32 * 1024);
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
                        .stack_size(32 * 1024);
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
    //执行开始逻辑
    room.start();
}

///选择占位
fn choice_location(rm: Arc<RwLock<RoomMgr>>, task: Task) {
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
    room.choice_location(user_id, None);
}

///选择占位
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
    room.choice_turn(user_id, None);
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

    let next_index = room.battle_data.next_turn_index;
    let next_user_id = room.battle_data.turn_orders[next_index];
    if next_user_id != user_id {
        return;
    }
    //如果玩家啥都没做，就T出房间
    if room.battle_data.turn_action.actions.is_empty() {
        room.remove_member(MemberLeaveNoticeType::Kicked as u8, &user_id);
    }
}
