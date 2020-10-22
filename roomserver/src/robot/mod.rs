use crate::robot::goal_think::GoalThink;
use crate::robot::robot_action::RobotStatusAction;
use crate::robot::robot_status::RobotStatus;
use crate::room::character::BattleCharacter;

pub mod goal_evaluator;
pub mod goal_think;
pub mod robot_action;
pub mod robot_status;
pub mod robot_task_mgr;

#[derive(Default)]
pub struct RobotData {
    pub goal_think: GoalThink,
    pub robot_status: Option<Box<dyn RobotStatusAction>>, //状态,
}

impl RobotData {
    pub fn thinking_do_something(&self, cter: &BattleCharacter) {
        self.goal_think.arbitrate(cter);
    }
}

impl Clone for RobotData {
    fn clone(&self) -> Self {
        RobotData::default()
    }
}
