use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_skill::can_use_skill;
use crate::robot::robot_status::skip_action::SkipRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

use super::buy_goal_evaluator::check_buy;

#[derive(Default)]
pub struct SkipGoalEvaluator {
    // desirability: AtomicCell<u32>,
}

impl GoalEvaluator for SkipGoalEvaluator {
    fn calculate_desirability(&self, robot: &BattlePlayer) -> u32 {
        if !robot.get_current_cter().map_cell_index_is_choiced() {
            return 0;
        }
        unsafe {
            let battle_data = robot
                .robot_data
                .as_ref()
                .unwrap()
                .battle_data
                .as_ref()
                .unwrap();
            let robot_index = robot.get_current_cter_index();
            let market_cell_index = battle_data.tile_map.market_cell.0;
            let is_at_market = market_cell_index == robot_index;
            let robot_data = robot.robot_data.as_ref().unwrap();
            let res = check_buy(robot, robot_data.temp_id);
            let can_buy = !res.is_empty() && is_at_market;
            let no_move_points = robot.flow_data.residue_movement_points == 0;
            let is_can_attack = robot.is_can_attack();
            let can_use_skill = can_use_skill(battle_data, robot);
            //如果什么都干不了了，则结束turn期望值拉满
            if no_move_points && !is_can_attack && !can_use_skill && !can_buy {
                return 100;
            }
            0
        }
    }

    fn set_status(
        &self,
        robot: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *mut BattleData,
    ) {
        let mut res = SkipRobotAction::new(battle_data, sender);
        res.cter_id = robot.get_cter_temp_id();
        res.robot_id = robot.get_user_id();
        res.temp_id = robot.robot_data.as_ref().unwrap().temp_id;
        robot.change_robot_status(Box::new(res));
    }
}
