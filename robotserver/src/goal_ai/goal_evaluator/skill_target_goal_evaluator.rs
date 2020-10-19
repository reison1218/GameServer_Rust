use crate::goal_ai::cter::Cter;
use crate::goal_ai::goal_evaluator::GoalEvaluator;
use crossbeam::atomic::AtomicCell;

#[derive(Default)]
pub struct SkillTargetGoalEvaluator {
    desirability: AtomicCell<u32>,
}

impl SkillTargetGoalEvaluator {
    pub fn new(desirability: u32) -> Self {
        let mut at = SkillTargetGoalEvaluator::default();
        at.desirability = AtomicCell::new(desirability);
        at
    }
}

impl GoalEvaluator for SkillTargetGoalEvaluator {
    fn calculate_desirability(&self) -> u32 {
        0
    }

    fn set_goal(&self, cter: &Cter) {
        println!("向SkillTargetGoalEvaluator设置goal");
        cter.goal_think.add_skill_target();
    }
}
