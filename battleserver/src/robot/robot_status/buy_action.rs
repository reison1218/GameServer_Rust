use crossbeam::channel::Sender;
use log::{error, info, warn};
use tools::cmd_code::BattleCode;

use crate::{
    battle::battle::BattleData,
    robot::{robot_action::RobotStatusAction, robot_task_mgr::RobotTask, RobotActionType},
};

use super::RobotStatus;

#[derive(Default)]
pub struct BuyRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub battle_data: Option<*const BattleData>,
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

    pub fn new(battle_data: *const BattleData, sender: Sender<RobotTask>) -> Self {
        let mut attack_action = BuyRobotAction::default();
        attack_action.battle_data = Some(battle_data);
        attack_action.sender = Some(sender);
        attack_action
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
        let res = self.get_battle_data_ref();
        if res.is_none() {
            warn!("the *const BattleData is null!");
            return;
        }
        let res = res.unwrap();
        let robot = res.get_battle_player(Some(self.robot_id), true);
        if let Err(_) = robot {
            return;
        }
        self.send_2_battle(1, RobotActionType::Buy, BattleCode::Action);
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

    fn send_2_battle(
        &self,
        merchandise_id: usize,
        robot_action_type: RobotActionType,
        cmd: BattleCode,
    ) {
        let mut robot_task = RobotTask::default();
        robot_task.action_type = robot_action_type;
        let mut map = serde_json::Map::new();
        map.insert(
            "user_id".to_owned(),
            crate::JsonValue::from(self.get_robot_id()),
        );
        map.insert(
            "merchandise_id".to_owned(),
            crate::JsonValue::from(merchandise_id),
        );
        map.insert("cmd".to_owned(), crate::JsonValue::from(cmd.into_u32()));
        robot_task.data = crate::JsonValue::from(map);
        let res = self.get_sender().send(robot_task);
        if let Err(e) = res {
            error!("{:?}", e);
        }
    }
}
