use crate::battle::battle::BattleData;
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::use_skill_action::UseSkillRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::character::BattleCharacter;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct UseSkillGoalEvaluator {
    desirability: AtomicCell<u32>,
}

impl GoalEvaluator for UseSkillGoalEvaluator {
    fn calculate_desirability(&self, cter: &BattleCharacter) -> u32 {
        if cter.can_use_skill() {
            return 100;
        }
        0
    }

    fn set_status(
        &self,
        cter: &BattleCharacter,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let aa = UseSkillRobotAction::new(battle_data, sender);
        cter.change_status(Box::new(aa));
    }
}
