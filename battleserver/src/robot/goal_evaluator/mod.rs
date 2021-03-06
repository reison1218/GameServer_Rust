pub mod attack_goal_evaluator;
pub mod buy_goal_evaluator;
pub mod choice_index_goal_evaluator;
pub mod open_cell_goal_evaluator;
pub mod skip_goal_evaluator;
pub mod unlock_goal_evaluator;
pub mod use_item_goal_evaluator;
pub mod use_skill_goal_evaluator;
use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

///评估trait
pub trait GoalEvaluator: Send + Sync + 'static {
    ///计算期望值
    fn calculate_desirability(&self, robot: &BattlePlayer) -> u32;

    ///设置状态
    fn set_status(
        &self,
        robot: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *mut BattleData,
    );
}
