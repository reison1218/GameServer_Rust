use super::*;
use crate::robot::RobotActionType;
use log::warn;
use tools::cmd_code::BattleCode;

#[derive(Default)]
pub struct AttackRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub battle_data: Option<*const BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

get_mut_ref!(AttackRobotAction);

impl AttackRobotAction {
    pub fn get_battle_data_ref(&self) -> Option<&BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }
            Some(self.battle_data.unwrap().as_ref().unwrap())
        }
    }

    pub fn new(battle_data: *const BattleData, sender: Sender<RobotTask>) -> Self {
        let mut attack_action = AttackRobotAction::default();
        attack_action.battle_data = Some(battle_data);
        attack_action.sender = Some(sender);
        attack_action
    }
}

impl RobotStatusAction for AttackRobotAction {
    fn set_sender(&mut self, sender: Sender<RobotTask>) {
        self.sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入攻击状态！", self.robot_id);
        self.execute();
    }

    fn execute(&self) {
        let res = self.get_battle_data_ref();
        if res.is_none() {
            warn!("the *const BattleData is null!");
            return;
        }
        let res = res.unwrap();
        let mut target_index: usize = 0;
        let mut cter_hp_max: i16 = 0;
        for battle_player in res.battle_player.values() {
            if battle_player.get_cter_id() == self.cter_id {
                continue;
            }
            if battle_player.cter.base_attr.hp > cter_hp_max {
                cter_hp_max = battle_player.cter.base_attr.hp;
                target_index = battle_player.get_map_cell_index();
            }
        }
        self.send_2_battle(target_index, RobotActionType::Attack, BattleCode::Action);
    }

    fn exit(&self) {
        unimplemented!()
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
