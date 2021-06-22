use std::time::Duration;

use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::choice_index_action::ChoiceIndexRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct ChoiceIndexGoalEvaluator {
    // desirability: AtomicCell<u32>,
}

impl GoalEvaluator for ChoiceIndexGoalEvaluator {
    fn calculate_desirability(&self, battle_player: &BattlePlayer) -> u32 {
        std::thread::sleep(Duration::from_secs(2));
        //如果没有选择站位，则期望值拉满
        if battle_player.cter.index_data.map_cell_index.is_none() {
            return 100;
        }
        0
    }

    fn set_status(
        &self,
        cter: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let aa = ChoiceIndexRobotAction::new(battle_data, sender);
        cter.change_robot_status(Box::new(aa));
    }
}
