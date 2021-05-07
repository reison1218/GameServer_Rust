use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::open_cell_action::OpenCellRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct OpenCellGoalEvaluator {
    // desirability: AtomicCell<u32>,
}

impl GoalEvaluator for OpenCellGoalEvaluator {
    fn calculate_desirability(&self, cter: &BattlePlayer) -> u32 {
        //如果可以翻地图块，则返回期望值10
        if cter.flow_data.residue_movement_points > 0 {
            return 10;
        }
        0
    }

    fn set_status(
        &self,
        cter: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let oa = OpenCellRobotAction::new(battle_data, sender);
        cter.change_robot_status(Box::new(oa));
    }
}
