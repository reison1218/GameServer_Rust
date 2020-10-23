pub mod attack_goal_evaluator;
pub mod open_cell_goal_evaluator;
use crate::room::character::BattleCharacter;

///评估trait
pub trait GoalEvaluator: Send + Sync + 'static {
    ///计算期望值
    fn calculate_desirability(&self, cter: &BattleCharacter) -> u32;

    ///设置评估
    fn set_status(&self, cter: &BattleCharacter);
}
