use crate::battle::battle::BattleData;
use crate::battle::battle_enum::ActionType;
use crate::robot::robot_action::RobotStatusAction;
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::character::BattleCharacter;
use crossbeam::channel::Sender;
use log::info;
use log::kv::ToValue;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use protobuf::Message;
use rand::Rng;
use rayon::iter::IntoParallelRefIterator;
use serde_json::{Map, Value};
use std::borrow::Borrow;
use std::sync::Arc;
use tools::cmd_code::RoomCode;
use tools::get_mut_ref;
use tools::macros::GetMutRef;
use tools::protos::battle::C_ACTION;
use tools::util::packet::Packet;

///pos操作类型
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RobotStatus {
    None = 0,
}

impl RobotStatus {
    pub fn into_u8(self) -> u8 {
        let value: u8 = self.into();
        value
    }
}

impl Default for RobotStatus {
    fn default() -> Self {
        RobotStatus::None
    }
}

#[derive(Default)]
pub struct Attack {
    pub robot_id: u32,
    pub cter_id: u32,
    pub battle_data: Option<*const BattleData>,
    pub status: RobotStatus,
    pub sender: Option<crossbeam::channel::Sender<RobotTask>>,
}

get_mut_ref!(Attack);

impl Attack {
    pub fn get_battle_data_ref(&self) -> &BattleData {
        unsafe { self.battle_data.unwrap().as_ref().unwrap() }
    }
}

impl RobotStatusAction for Attack {
    fn set_sender(&self, sender: Sender<RobotTask>) {
        self.get_mut_ref().sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入攻击状态！", self.cter_id);
    }

    fn execute(&self) {
        let res = self.get_battle_data_ref();
        let mut target_index: usize = 0;
        let mut cter_hp_max: i16 = 0;
        for cter in res.battle_cter.values() {
            if cter.get_cter_id() == self.cter_id {
                continue;
            }
            if cter.base_attr.hp > cter_hp_max {
                cter_hp_max = cter.base_attr.hp;
                target_index = cter.get_map_cell_index();
            }
        }
        //创建机器人任务执行普通攻击
        let mut robot_task = RobotTask::default();
        robot_task.cmd = ActionType::Attack.into();
        let mut map = Map::new();
        map.insert("user_id".to_owned(), Value::from(self.robot_id));
        map.insert("target_index".to_owned(), Value::from(target_index));
        map.insert("cmd".to_owned(), Value::from(RoomCode::Action.into_u32()));
        self.sender.as_ref().unwrap().send(robot_task);
    }

    fn exit(&self) {
        unimplemented!()
    }

    fn get_status(&self) -> RobotStatus {
        self.status
    }
}

#[derive(Default)]
pub struct OpenCell {
    pub robot_id: u32,
    pub cter_id: u32,
    pub battle_data: Option<Arc<*const BattleData>>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

impl OpenCell {
    pub fn get_battle_data_ref(&self) -> &BattleData {
        unsafe {
            let ptr = self.battle_data.as_ref().unwrap().as_ref();
            let battle_data_ref = ptr.as_ref().unwrap();
            battle_data_ref
        }
    }
}

get_mut_ref!(OpenCell);

impl RobotStatusAction for OpenCell {
    fn set_sender(&self, sender: Sender<RobotTask>) {
        self.get_mut_ref().sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        self.execute();
    }

    fn execute(&self) {
        let battle_data = self.get_battle_data_ref();
        if battle_data.tile_map.un_pair_map.is_empty() {
            return;
        }
        let mut v = Vec::new();
        for key in battle_data.tile_map.un_pair_map.keys() {
            v.push(*key);
        }
        let mut rand = rand::thread_rng();
        let index = rand.gen_range(0, v.len());

        //创建机器人任务执行普通攻击
        let mut robot_task = RobotTask::default();
        robot_task.cmd = ActionType::Open.into();
        let mut map = Map::new();
        map.insert("user_id".to_owned(), Value::from(self.robot_id));
        map.insert("value".to_owned(), Value::from(index));
        map.insert("cmd".to_owned(), Value::from(RoomCode::Action.into_u32()));
        self.sender.as_ref().unwrap().send(robot_task);
    }

    fn exit(&self) {
        unimplemented!()
    }

    fn get_status(&self) -> RobotStatus {
        self.status
    }
}
