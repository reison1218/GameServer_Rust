use super::*;
use crate::robot::RobotActionType;
use log::warn;
use tools::cmd_code::BattleCode;

#[derive(Default)]
pub struct ChoiceIndexRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub battle_data: Option<*const BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

get_mut_ref!(ChoiceIndexRobotAction);

impl ChoiceIndexRobotAction {
    pub fn get_battle_data_ref(&self) -> Option<&BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }
            Some(self.battle_data.unwrap().as_ref().unwrap())
        }
    }

    pub fn new(battle_data: *const BattleData, sender: Sender<RobotTask>) -> Self {
        let mut attack_action = ChoiceIndexRobotAction::default();
        attack_action.battle_data = Some(battle_data);
        attack_action.sender = Some(sender);
        attack_action
    }
}

impl RobotStatusAction for ChoiceIndexRobotAction {
    fn set_sender(&self, sender: Sender<RobotTask>) {
        self.get_mut_ref().sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入选择站位状态！", self.robot_id);
        self.execute();
    }

    fn execute(&self) {
        let battle_data = self.get_battle_data_ref();
        if battle_data.is_none() {
            warn!("the point *const BattleData is null!");
            return;
        }
        let battle_data = battle_data.unwrap();
        let mut v = Vec::new();
        let size = battle_data.tile_map.un_pair_map.len();
        let mut index;
        for i in battle_data.tile_map.un_pair_map.keys() {
            index = *i;
            let map_cell = battle_data.tile_map.map_cells.get(index).unwrap();
            if map_cell.user_id > 0 {
                continue;
            }
            v.push(index);
        }
        let mut rand = rand::thread_rng();
        let index = rand.gen_range(0..size);

        //创建机器人任务执行选择站位
        self.send_2_battle(index, RobotActionType::ChoiceIndex, BattleCode::ChoiceIndex);
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
