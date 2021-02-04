use super::*;
use crate::robot::RobotActionType;
use tools::cmd_code::BattleCode;

#[derive(Default)]
pub struct SkipRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub battle_data: Option<*const BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

get_mut_ref!(SkipRobotAction);

impl SkipRobotAction {
    pub fn get_battle_data_ref(&self) -> &BattleData {
        unsafe { self.battle_data.unwrap().as_ref().unwrap() }
    }

    pub fn new(battle_data: *const BattleData, sender: Sender<RobotTask>) -> Self {
        let mut attack_action = SkipRobotAction::default();
        attack_action.battle_data = Some(battle_data);
        attack_action.sender = Some(sender);
        attack_action
    }
}

impl RobotStatusAction for SkipRobotAction {
    fn set_sender(&self, sender: Sender<RobotTask>) {
        self.get_mut_ref().sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入攻击状态！", self.cter_id);
        self.execute();
    }

    fn execute(&self) {
        let res = self.get_battle_data_ref();
        let mut target_index: usize = 0;
        let mut cter_hp_max: i16 = 0;
        for cter in res.battle_cter.values() {
            if cter.get_cter_id() == self.cter_id {
                continue;
            }
            if cter.base_attr.hp > cter_hp_max {
                cter_hp_max = cter.base_attr.hp;
                target_index = cter.get_map_cell_index();
            }
        }
        //创建机器人任务执行普通攻击
        self.send_2_battle(target_index, RobotActionType::Skip, BattleCode::Action);
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
