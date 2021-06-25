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
        let robot_data = cter.robot_data.as_ref().unwrap();
        let pair_index = robot_data.can_pair_index();

        if pair_index.is_some() && cter.flow_data.residue_movement_points > 0 {
            return 70;
        }
        50
    }

    fn set_status(
        &self,
        robot: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let mut res = OpenCellRobotAction::new(battle_data, sender);
        res.cter_id = robot.get_cter_id();
        res.robot_id = robot.get_user_id();
        robot.change_robot_status(Box::new(res));
    }
}
