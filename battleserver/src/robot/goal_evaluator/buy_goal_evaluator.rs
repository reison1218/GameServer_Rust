use super::GoalEvaluator;

use crate::battle::battle::BattleData;
use crate::battle::battle_player::BattlePlayer;
use crate::robot::robot_status::buy_action::BuyRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
// use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Sender;
use rand::Rng;

#[derive(Default)]
pub struct BuyGoalEvaluator {
    // desirability: AtomicCell<u32>,
}

impl GoalEvaluator for BuyGoalEvaluator {
    fn calculate_desirability(&self, robot: &BattlePlayer) -> u32 {
        if !robot.cter.map_cell_index_is_choiced() {
            return 0;
        }
        let robot_index = robot.get_map_cell_index();
        let robot_data = robot.robot_data.as_ref().unwrap();
        let battle_data = robot.robot_data.as_ref().unwrap().battle_data;
        unsafe {
            let battle_data = battle_data.as_ref().unwrap();
            let market_cell_index = battle_data.tile_map.market_cell.0;
            let is_at_market = market_cell_index == robot_index;
            let market = battle_data
                .tile_map
                .map_cells
                .get(market_cell_index)
                .unwrap();
            //判断商店上面的人是否有稳如泰山被动
            if market.user_id != robot.get_user_id() {
                let player = battle_data.battle_player.get(&market.user_id);
                if let Some(player) = player {
                    if !player.can_be_move() {
                        return 0;
                    }
                }
            }

            let res = check_buy(robot, robot_data.temp_id);
            //只能买一个的时候，并且不在商店，并且有行动点数的时候
            if res.len() == 1 && !is_at_market && robot.flow_data.residue_movement_points > 0 {
                let mut rand = rand::thread_rng();
                let res = rand.gen_range(40..56);
                return res;
            } else if res.len() > 1 && !is_at_market && robot.flow_data.residue_movement_points > 0
            {
                //能买多个，并且不在商店，并且有行动点数的时候
                return 60;
            } else if !res.is_empty() && is_at_market {
                //在商店，并且可以购买的时候
                return 90;
            }
        }
        0
    }

    fn set_status(
        &self,
        robot: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *mut BattleData,
    ) {
        let mut res = BuyRobotAction::new(battle_data, sender);
        res.cter_id = robot.get_cter_id();
        res.robot_id = robot.get_user_id();
        res.temp_id = robot.robot_data.as_ref().unwrap().temp_id;
        robot.change_robot_status(Box::new(res));
    }
}

fn get_could_buy(battle_player: &BattlePlayer, merchandise_id: u32) -> bool {
    let merchandise_temp = crate::TEMPLATES.merchandise_temp_mgr();
    let temp = merchandise_temp.get_temp(&merchandise_id);
    if let Err(_) = temp {
        return false;
    }
    let merchandise_temp = temp.unwrap();
    let turn_limit_buy_times = merchandise_temp.turn_limit_buy_times;
    //校验是否可以购买
    let buy_times = battle_player
        .merchandise_data
        .get_turn_buy_times(merchandise_id);
    if buy_times >= turn_limit_buy_times {
        return false;
    }

    let price = merchandise_temp.price;
    if battle_player.gold < price {
        return false;
    }
    true
}

pub fn check_buy(battle_player: &BattlePlayer, robot_temp_id: u32) -> Vec<u32> {
    let temp = crate::TEMPLATES
        .robot_temp_mgr()
        .get_temp_ref(&robot_temp_id);
    if let None = temp {
        return vec![];
    }
    let mut res_v = vec![];
    let temp = temp.unwrap();
    for &merchandise in temp.merchandises.iter() {
        let res = get_could_buy(battle_player, merchandise);
        if !res {
            continue;
        }
        res_v.push(merchandise);
    }
    res_v
}
