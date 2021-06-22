pub mod attack_action;
pub mod buy_action;
pub mod choice_index_action;
pub mod open_cell_action;
pub mod skip_action;
pub mod use_item_action;
pub mod use_skill_action;

use crate::battle::battle::BattleData;
use crate::robot::robot_action::RobotStatusAction;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;
use rand::Rng;

use log::info;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use tools::get_mut_ref;

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
