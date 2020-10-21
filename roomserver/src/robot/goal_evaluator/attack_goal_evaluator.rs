use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::Attack;
use crate::room::character::BattleCharacter;
use crossbeam::atomic::AtomicCell;

#[derive(Default)]
pub struct AttackTargetGoalEvaluator {
    desirability: AtomicCell<u32>,
}

impl GoalEvaluator for AttackTargetGoalEvaluator {
    fn calculate_desirability(&self) -> u32 {
        1
    }

    fn set_status(&self, cter: &mut BattleCharacter) {
        cter.set_robot_action(Box::new(Attack::default()));
    }
}
