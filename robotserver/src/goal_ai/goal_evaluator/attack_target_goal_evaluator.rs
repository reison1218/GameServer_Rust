use crate::goal_ai::cter::Cter;
use crate::goal_ai::goal_evaluator::GoalEvaluator;
use crate::goal_ai::goals::goal::Goal;
use crossbeam::atomic::AtomicCell;

#[derive(Default)]
pub struct AttackTargetGoalEvaluator {
    desirability: AtomicCell<u32>,
}

impl AttackTargetGoalEvaluator {
    pub fn new(desirability: u32) -> Self {
        let mut at = AttackTargetGoalEvaluator::default();
        at.desirability = AtomicCell::new(desirability);
        at
    }
}

impl GoalEvaluator for AttackTargetGoalEvaluator {
    fn calculate_desirability(&self) -> u32 {
        0
    }

    fn set_goal(&self, cter: &Cter) {
        println!("向AttackTargetGoalEvaluator设置goal");
        cter.goal_think.add_attack_target();
    }
}
