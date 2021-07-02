use super::*;
use crate::robot::RobotActionType;
use log::warn;
use tools::cmd_code::BattleCode;

#[derive(Default)]
pub struct SkipRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub temp_id: u32,
    pub battle_data: Option<*mut BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

get_mut_ref!(SkipRobotAction);

impl SkipRobotAction {
    pub fn new(battle_data: *mut BattleData, sender: Sender<RobotTask>) -> Self {
        let mut attack_action = SkipRobotAction::default();
        attack_action.battle_data = Some(battle_data);
        attack_action.sender = Some(sender);
        attack_action
    }
    pub fn get_battle_data_mut_ref(&self) -> Option<&mut BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }

            Some(self.battle_data.unwrap().as_mut().unwrap())
        }
    }
}

impl RobotStatusAction for SkipRobotAction {
    fn set_sender(&mut self, sender: Sender<RobotTask>) {
        self.sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入跳过状态！", self.robot_id);
        self.execute();
    }

    fn execute(&self) {
        let battle_data = self.get_battle_data_mut_ref();
        if battle_data.is_none() {
            warn!("the point *const BattleData is null!");
            return;
        }
        //创建机器人任务执行结束turn
        self.send_2_battle(0, RobotActionType::Skip, BattleCode::Action);
    }

    fn exit(&self) {
        // info!("robot:{} 退出跳过状态！", self.robot_id);
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
