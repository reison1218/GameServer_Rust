use super::*;
use crate::robot::RobotActionType;
use log::warn;
use tools::cmd_code::BattleCode;

#[derive(Default)]
pub struct UnlockRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub temp_id: u32,
    pub battle_data: Option<*mut BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

get_mut_ref!(UnlockRobotAction);

impl UnlockRobotAction {
    pub fn get_battle_data_mut_ref(&self) -> Option<&mut BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }

            Some(self.battle_data.unwrap().as_mut().unwrap())
        }
    }

    pub fn new(battle_data: *mut BattleData, sender: Sender<RobotTask>) -> Self {
        let mut attack_action = UnlockRobotAction::default();
        attack_action.battle_data = Some(battle_data);
        attack_action.sender = Some(sender);
        attack_action
    }
}

impl RobotStatusAction for UnlockRobotAction {
    fn set_sender(&mut self, sender: Sender<RobotTask>) {
        self.sender = Some(sender);
    }

    fn get_cter_temp_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入解除锁定状态！", self.robot_id);
        self.execute();
    }

    fn execute(&self) {
        let res = self.get_battle_data_mut_ref();
        if res.is_none() {
            warn!("the *const BattleData is null!");
            return;
        }
        let battle_data = res.unwrap();

        let battle_player = battle_data.battle_player.get(&self.robot_id).unwrap();
        let target_index: usize = battle_player.get_current_cter_index();
        self.send_2_battle(target_index, RobotActionType::Unlock, BattleCode::Action);
    }

    fn exit(&self) {
        // info!("robot:{} 退出攻击状态！", self.robot_id);
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
