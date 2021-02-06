pub mod attack_goal_evaluator;
pub mod choice_index_goal_evaluator;
pub mod open_cell_goal_evaluator;
pub mod skip_goal_evaluator;
pub mod use_item_goal_evaluator;
pub mod use_skill_goal_evaluator;
use crate::battle::battle::BattleData;
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::character::BattleCharacter;
use crossbeam::channel::Sender;

///评估trait
pub trait GoalEvaluator: Send + Sync + 'static {
    ///计算期望值
    fn calculate_desirability(&self, cter: &BattleCharacter) -> u32;

    ///设置状态
    fn set_status(
        &self,
        cter: &BattleCharacter,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    );
}
