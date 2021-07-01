use super::*;
use crate::{robot::RobotActionType, room::map_data::MapCellType};
use log::{info, warn};
use tools::cmd_code::BattleCode;

#[derive(Default)]
pub struct ChoiceIndexRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub temp_id: u32,
    pub battle_data: Option<*mut BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

get_mut_ref!(ChoiceIndexRobotAction);

impl ChoiceIndexRobotAction {
    pub fn get_battle_data_mut_ref(&self) -> Option<&mut BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }

            Some(self.battle_data.unwrap().as_mut().unwrap())
        }
    }

    pub fn new(battle_data: *mut BattleData, sender: Sender<RobotTask>) -> Self {
        let mut choice_index_action = ChoiceIndexRobotAction::default();
        choice_index_action.battle_data = Some(battle_data);
        choice_index_action.sender = Some(sender);
        choice_index_action
    }
}

impl RobotStatusAction for ChoiceIndexRobotAction {
    fn set_sender(&mut self, sender: Sender<RobotTask>) {
        self.sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入选择站位状态！", self.robot_id);
        self.execute();
    }

    fn execute(&self) {
        let battle_data = self.get_battle_data_mut_ref();
        if battle_data.is_none() {
            warn!("the point *const BattleData is null!");
            return;
        }
        let battle_data = battle_data.unwrap();
        let mut v = Vec::new();
        for (&index, _) in battle_data.tile_map.un_pair_map.iter() {
            let map_cell = battle_data.tile_map.map_cells.get(index).unwrap();
            if map_cell.user_id > 0 {
                continue;
            }
            if map_cell.is_world() {
                continue;
            }
            if map_cell.id <= MapCellType::UnUse.into_u32() {
                continue;
            }
            v.push(index);
        }
        let mut rand = rand::thread_rng();
        let res = rand.gen_range(0..v.len());
        let index = v.remove(res);
        //创建机器人任务执行选择站位
        self.send_2_battle(index, RobotActionType::ChoiceIndex, BattleCode::ChoiceIndex);
    }

    fn exit(&self) {
        // info!("robot:{} 退出选择展位状态！", self.robot_id);
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
