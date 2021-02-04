pub mod attack_action;
pub mod choice_index_action;
pub mod open_cell_action;
pub mod robot_status;
pub mod skip_action;

use crate::battle::battle::BattleData;
use crate::robot::robot_action::RobotStatusAction;
use crate::robot::robot_status::robot_status::RobotStatus;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;
use rand::Rng;

use log::info;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use tools::get_mut_ref;
use tools::macros::GetMutRef;
