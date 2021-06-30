use crossbeam::channel::Sender;
use log::{error, info, warn};
use tools::cmd_code::BattleCode;

use crate::{
    battle::battle::BattleData,
    robot::{
        goal_evaluator::buy_goal_evaluator::check_buy, robot_action::RobotStatusAction,
        robot_helper::modify_robot_state, robot_task_mgr::RobotTask, RobotActionType,
    },
};

use super::RobotStatus;

#[derive(Default)]
pub struct BuyRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub temp_id: u32,
    pub battle_data: Option<*mut BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

impl BuyRobotAction {
    pub fn get_battle_data_ref(&self) -> Option<&BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }
            Some(self.battle_data.unwrap().as_ref().unwrap())
        }
    }

    pub fn get_battle_data_mut_ref(&self) -> Option<&mut BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }

            Some(self.battle_data.unwrap().as_mut().unwrap())
        }
    }

    pub fn new(battle_data: *mut BattleData, sender: Sender<RobotTask>) -> Self {
        let mut attack_action = BuyRobotAction::default();
        attack_action.battle_data = Some(battle_data);
        attack_action.sender = Some(sender);
        attack_action
    }

    fn send_2_battle(
        &self,
        target_index: usize,
        merchandise_id: usize,
        robot_action_type: RobotActionType,
        cmd: BattleCode,
    ) {
        let mut robot_task = RobotTask::default();
        robot_task.action_type = robot_action_type;
        robot_task.robot_id = self.robot_id;
        let mut map = serde_json::Map::new();
        map.insert(
            "merchandise_id".to_owned(),
            crate::JsonValue::from(merchandise_id),
        );
        map.insert("value".to_owned(), crate::JsonValue::from(target_index));
        map.insert("cmd".to_owned(), crate::JsonValue::from(cmd.into_u32()));
        robot_task.data = crate::JsonValue::from(map);
        let res = self.get_sender().send(robot_task);
        if let Err(e) = res {
            error!("{:?}", e);
        }
    }
}

impl RobotStatusAction for BuyRobotAction {
    fn set_sender(&mut self, sender: Sender<RobotTask>) {
        self.sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入购物状态！", self.robot_id);
        self.execute();
    }

    fn execute(&self) {
        let res = self.get_battle_data_mut_ref();
        if res.is_none() {
            warn!("the *const BattleData is null!");
            return;
        }
        let battle_data = res.unwrap();
        let robot = battle_data.get_battle_player(Some(self.robot_id), true);
        if let Err(_) = robot {
            return;
        }
        let robot = robot.unwrap();

        let market_cell_index = battle_data.tile_map.market_cell.0;
        let is_at_market = market_cell_index == robot.get_map_cell_index();

        let res = check_buy(robot, self.temp_id);
        if !res.is_empty() && is_at_market {
            let merchandise_id = *res.get(0).unwrap() as usize;
            modify_robot_state(self.robot_id, battle_data);
            self.send_2_battle(0, merchandise_id, RobotActionType::Buy, BattleCode::Buy);
        } else if !res.is_empty() && !is_at_market && robot.flow_data.residue_movement_points > 0 {
            modify_robot_state(self.robot_id, battle_data);
            self.send_2_battle(
                market_cell_index,
                0,
                RobotActionType::Open,
                BattleCode::Action,
            );
        }
    }

    fn exit(&self) {
        // info!("robot:{} 退出购买状态！", self.robot_id);
    }

    fn get_status(&self) -> RobotStatus {
        self.status
    }

    fn get_robot_id(&self) -> u32 {
        self.robot_id
    }

    fn get_sender(&self) -> &Sender<RobotTask> {
        self.sender.as_ref().unwrap()
    }
}
