use crate::robot::goal_think::GoalThink;
use crate::robot::robot_action::RobotStatusAction;
use crate::room::character::BattleCharacter;

pub mod goal_evaluator;
pub mod goal_think;
pub mod robot_action;
pub mod robot_status;
pub mod robot_task_mgr;
use crate::battle::battle::BattleData;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct RobotData {
    pub goal_think: GoalThink,
    pub robot_status: Option<Box<dyn RobotStatusAction>>, //状态,
    pub sender: Option<Sender<RobotTask>>,
}

impl RobotData {
    pub fn thinking_do_something(&self, cter: &BattleCharacter, battle_data: *const BattleData) {
        self.goal_think
            .arbitrate(cter, self.sender.as_ref().unwrap().clone(), battle_data);
    }
}

impl Clone for RobotData {
    fn clone(&self) -> Self {
        RobotData::default()
    }
}
