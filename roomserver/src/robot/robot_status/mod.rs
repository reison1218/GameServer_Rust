pub mod attack_action;
pub mod open_cell_action;
pub mod robot_status;

use crate::battle::battle::BattleData;
use crate::battle::battle_enum::ActionType;
use crate::robot::robot_action::RobotStatusAction;
use crate::robot::robot_status::robot_status::RobotStatus;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;
use rand::Rng;
use serde_json::Value;
use tools::cmd_code::RoomCode;

use log::{error, info};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde_json::Map;
use tools::get_mut_ref;
use tools::macros::GetMutRef;
