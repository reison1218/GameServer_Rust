use crate::battle::battle_enum::ActionType;
use crate::{JsonValue, Lock, ROBOT_SCHEDULED_MGR};
use async_std::task::block_on;
use log::error;
use log::info;
use protobuf::Message;
use std::time::Duration;
use tools::macros::GetMutRef;
use tools::protos::battle::{C_ACTION, C_BUY, C_CHOOSE_INDEX};
use tools::util::packet::Packet;

use super::RobotActionType;

#[derive(Debug, Clone)]
pub struct RobotTask {
    pub action_type: RobotActionType, //要执行的命令
    pub robot_id: u32,
    pub delay: u64,      //要延迟执行的时间
    pub data: JsonValue, //数据
}

impl Default for RobotTask {
    fn default() -> Self {
        RobotTask {
            action_type: RobotActionType::Attack,
            robot_id: 0,
            delay: 5000,
            data: JsonValue::default(),
        }
    }
}

///初始化定时执行任务
pub fn robot_init_timer(bm: Lock) {
    let m = move || {
        let (sender, rec) = crossbeam::channel::bounded(1024);
        let mut lock = block_on(bm.lock());
        lock.robot_task_sender = Some(sender);
        std::mem::drop(lock);
        loop {
            let res = rec.recv();
            if let Err(e) = res {
                error!("{:?}", e);
                continue;
            }
            let task = res.unwrap();
            let delay = task.delay;

            let task_cmd = task.action_type;
            let rm_clone = bm.clone();
            let fnc = match task_cmd {
                RobotActionType::Attack => attack,
                RobotActionType::Skill => use_skill,
                RobotActionType::Open => open_cell,
                RobotActionType::Skip => skip_turn,
                RobotActionType::UseItem => use_item,
                RobotActionType::ChoiceIndex => choice_index,
                RobotActionType::Buy => buy,
                RobotActionType::Unlock => unlock,
                RobotActionType::None => none,
            };
            let m = move || fnc(rm_clone, task);
            ROBOT_SCHEDULED_MGR.execute_after(Duration::from_millis(delay), m);
        }
    };
    let timer_thread = std::thread::Builder::new().name("ROBOT_TIMER_THREAD".to_owned());
    let res = timer_thread.spawn(m);
    if let Err(e) = res {
        error!("{:?}", e);
    }
    info!("初始化定时器任务执行器成功!");
}

///机器人选择站位函数
pub fn choice_index(rm: Lock, task: RobotTask) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();
    let robot_id = task.robot_id;
    let target_index = map.get("target_index").unwrap().as_u64().unwrap() as u32;
    let cmd = map.get("cmd").unwrap().as_u64().unwrap() as u32;

    let mut packet = Packet::new(cmd, 0, robot_id);
    let mut proto = C_CHOOSE_INDEX::new();
    proto.set_index(target_index);
    packet.set_data(proto.write_to_bytes().unwrap().as_slice());

    let lock = block_on(rm.lock());
    //拿到BattleMgr的可变指针
    let rm_mut_ref = lock.get_mut_ref();
    let func = lock.cmd_map.get(&cmd).unwrap();
    func(rm_mut_ref, packet);
}

///普通攻击
pub fn buy(rm: Lock, task: RobotTask) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();
    let robot_id = task.robot_id;
    let merchandise_id = map.get("merchandise_id").unwrap().as_u64().unwrap() as u32;
    let cmd = map.get("cmd").unwrap().as_u64().unwrap() as u32;

    let mut packet = Packet::new(cmd, 0, robot_id);
    let mut ca = C_BUY::new();
    ca.merchandise_id = merchandise_id;
    packet.set_data(ca.write_to_bytes().unwrap().as_slice());
    //解锁,获得函数指针，执行普通攻击逻辑
    let lock = block_on(rm.lock());
    //拿到BattleMgr的可变指针
    let rm_mut_ref = lock.get_mut_ref();
    let func = lock.cmd_map.get(&cmd).unwrap();
    func(rm_mut_ref, packet);
}

pub fn unlock(rm: Lock, task: RobotTask) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();
    let robot_id = task.robot_id;
    let target_index = map.get("target_index").unwrap().as_u64().unwrap() as u32;
    let cmd = map.get("cmd").unwrap().as_u64().unwrap() as u32;
    let mut packet = Packet::new(cmd, 0, robot_id);
    let mut ca = C_ACTION::new();
    ca.target_index.push(target_index);
    ca.action_type = ActionType::EndShowMapCell.into_u32();
    packet.set_data(ca.write_to_bytes().unwrap().as_slice());
    //解锁,获得函数指针，执行普通攻击逻辑
    let lock = block_on(rm.lock());
    //拿到BattleMgr的可变指针
    let rm_mut_ref = lock.get_mut_ref();
    let func = lock.cmd_map.get(&cmd).unwrap();
    func(rm_mut_ref, packet);
}

///普通攻击
pub fn none(_: Lock, _: RobotTask) {}

///普通攻击
pub fn attack(rm: Lock, task: RobotTask) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();
    let robot_id = task.robot_id;
    let target_index = map.get("target_index").unwrap().as_u64().unwrap() as u32;
    let cmd = map.get("cmd").unwrap().as_u64().unwrap() as u32;

    let mut packet = Packet::new(cmd, 0, robot_id);
    let mut ca = C_ACTION::new();
    ca.target_index.push(target_index);
    ca.action_type = ActionType::Attack.into_u32();
    packet.set_data(ca.write_to_bytes().unwrap().as_slice());
    //解锁,获得函数指针，执行普通攻击逻辑
    let lock = block_on(rm.lock());
    //拿到BattleMgr的可变指针
    let rm_mut_ref = lock.get_mut_ref();
    let func = lock.cmd_map.get(&cmd).unwrap();
    func(rm_mut_ref, packet);
}

///打开地图块
pub fn open_cell(rm: Lock, task: RobotTask) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();
    let robot_id = task.robot_id;
    let value = map.get("value").unwrap().as_u64().unwrap() as u32;
    let cmd = map.get("cmd").unwrap().as_u64().unwrap() as u32;

    let mut packet = Packet::new(cmd, 0, robot_id);
    let mut ca = C_ACTION::new();
    ca.value = value;
    ca.action_type = ActionType::Open.into_u32();
    packet.set_data(ca.write_to_bytes().unwrap().as_slice());
    //解锁,获得函数指针，执行普通攻击逻辑
    let lock = block_on(rm.lock());
    //拿到BattleMgr的可变指针
    let rm_mut_ref = lock.get_mut_ref();
    let func = lock.cmd_map.get(&cmd).unwrap();
    func(rm_mut_ref, packet);
}

///跳过回合
pub fn skip_turn(rm: Lock, task: RobotTask) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();
    let user_id = task.robot_id;
    let cmd = map.get("cmd").unwrap().as_u64().unwrap() as u32;

    let mut packet = Packet::new(cmd, 0, user_id);
    let mut ca = C_ACTION::new();
    ca.action_type = ActionType::Skip.into_u32();
    packet.set_data(ca.write_to_bytes().unwrap().as_slice());
    //解锁,获得函数指针，执行普通攻击逻辑
    let lock = block_on(rm.lock());
    //拿到BattleMgr的可变指针
    let rm_mut_ref = lock.get_mut_ref();
    let func = lock.cmd_map.get(&cmd).unwrap();
    func(rm_mut_ref, packet);
}

///使用技能
pub fn use_skill(rm: Lock, task: RobotTask) {
    let json_value = task.data;
    let res = json_value.as_object();
    if res.is_none() {
        return;
    }
    let map = res.unwrap();
    let robot_id = task.robot_id;
    let target_index = map.get("target_index").unwrap().as_array().unwrap();
    let skill_id = map.get("skill_id").unwrap().as_u64().unwrap() as u32;
    let cmd = map.get("cmd").unwrap().as_u64().unwrap() as u32;

    let mut proto = C_ACTION::new();
    proto.action_type = ActionType::Skill.into_u32();
    for target in target_index {
        proto.target_index.push(target.as_u64().unwrap() as u32);
    }
    proto.value = skill_id;

    let mut packet = Packet::new(cmd, 0, robot_id);
    packet.set_data(proto.write_to_bytes().unwrap().as_slice());
    let lock = block_on(rm.lock());
    //拿到BattleMgr的可变指针
    let rm_mut_ref = lock.get_mut_ref();
    let func = lock.cmd_map.get(&cmd).unwrap();
    func(rm_mut_ref, packet);
}

///使用道具
pub fn use_item(_: Lock, _: RobotTask) {}
