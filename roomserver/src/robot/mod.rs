use crate::robot::goal_think::GoalThink;
use crate::robot::robot_action::RobotStatusAction;
use crate::robot::robot_status::RobotStatus;

pub mod goal_evaluator;
pub mod goal_think;
pub mod robot_action;
pub mod robot_status;

#[derive(Default)]
pub struct RobotData {
    pub goal_think: GoalThink,
    pub robot_status: Option<Box<dyn RobotStatusAction>>, //状态,
}

impl Clone for RobotData {
    fn clone(&self) -> Self {
        RobotData::default()
    }
}
