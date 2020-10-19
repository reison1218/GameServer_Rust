pub mod attack_target_goal_evaluator;
pub mod skill_target_goal_evaluator;
use crate::goal_ai::cter::Cter;

///评估trait
pub trait GoalEvaluator: Send + 'static {
    ///计算期望值
    fn calculate_desirability(&self) -> u32;

    ///设置评估
    fn set_goal(&self, cter: &Cter);
}
