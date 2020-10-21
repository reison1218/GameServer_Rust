pub mod attack_goal_evaluator;
pub mod open_cell_goal_evaluator;
use crate::robot::goal_evaluator::attack_goal_evaluator::AttackTargetGoalEvaluator;
use crate::room::character::BattleCharacter;

///评估trait
pub trait GoalEvaluator: Send + 'static {
    ///计算期望值
    fn calculate_desirability(&self) -> u32;

    ///设置评估
    fn set_status(&self, cter: &mut BattleCharacter);
}
