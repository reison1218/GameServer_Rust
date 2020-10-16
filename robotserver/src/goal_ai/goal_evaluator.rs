use crate::goal_ai::cter::Cter;
use crate::goal_ai::goal::Goal;

///评估trait
pub trait GoalEvaluator: Send + 'static {
    ///计算期望值
    fn calculate_desirability(&self) -> u32;

    ///设置评估
    fn set_goal(&self, cter: &Cter);
}
