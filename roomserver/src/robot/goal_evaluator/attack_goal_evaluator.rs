use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::Attack;
use crate::room::character::BattleCharacter;
use crossbeam::atomic::AtomicCell;

#[derive(Default)]
pub struct AttackTargetGoalEvaluator {
    desirability: AtomicCell<u32>,
}

impl GoalEvaluator for AttackTargetGoalEvaluator {
    fn calculate_desirability(&self, cter: &BattleCharacter) -> u32 {
        if cter.is_can_attack() {
            return 1;
        }
        0
    }

    fn set_status(&self, cter: &BattleCharacter) {
        cter.change_status(Box::new(Attack::default()));
    }
}
