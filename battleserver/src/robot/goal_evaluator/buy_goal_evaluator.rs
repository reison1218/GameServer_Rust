use super::GoalEvaluator;

use crate::battle::battle::BattleData;
use crate::battle::battle_player::BattlePlayer;
use crate::robot::robot_status::buy_action::BuyRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct BuyGoalEvaluator {
    //desirability: AtomicCell<u32>,
}

impl GoalEvaluator for BuyGoalEvaluator {
    fn calculate_desirability(&self, robot: &BattlePlayer) -> u32 {
        let robot_index = robot.get_map_cell_index();
        let battle_data = robot.robot_data.as_ref().unwrap().battle_data;
        unsafe {
            let battle_data = battle_data.as_ref().unwrap();
            let market_cell_index = battle_data.tile_map.market_cell.0;
            if market_cell_index != robot_index && robot.gold >= 20 {
                return 90;
            } else if robot.gold >= 20 {
                return 60;
            }
        }
        0
    }

    fn set_status(
        &self,
        robot: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let mut res = BuyRobotAction::new(battle_data, sender);
        res.cter_id = robot.get_cter_id();
        res.robot_id = robot.get_user_id();
        robot.change_robot_status(Box::new(res));
    }
}
