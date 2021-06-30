use super::*;
use crate::robot::RobotActionType;
use tools::cmd_code::BattleCode;

#[derive(Default)]
pub struct UseItemRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub temp_id: u32,
    pub battle_data: Option<*mut BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

get_mut_ref!(UseItemRobotAction);

impl UseItemRobotAction {
    pub fn new(battle_data: *mut BattleData, sender: Sender<RobotTask>) -> Self {
        let mut use_item_action = UseItemRobotAction::default();
        use_item_action.battle_data = Some(battle_data);
        use_item_action.sender = Some(sender);
        use_item_action
    }

    pub fn get_battle_data_mut_ref(&mut self) -> Option<&mut BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }

            Some(self.battle_data.unwrap().as_mut().unwrap())
        }
    }
}

impl RobotStatusAction for UseItemRobotAction {
    fn set_sender(&mut self, sender: Sender<RobotTask>) {
        self.sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入使用道具状态！", self.robot_id);
        self.execute();
    }

    fn execute(&self) {
        //todo 根据道具类型选举函数执行
        self.send_2_battle(0, RobotActionType::UseItem, BattleCode::Action);
    }

    fn exit(&self) {
        // info!("robot:{} 退出使用道具状态！", self.robot_id);
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
