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
    pub fn new(battle_data: *const BattleData, sender: Sender<RobotTask>) -> Self {
        let mut attack_action = SkipRobotAction::default();
        attack_action.battle_data = Some(battle_data);
        attack_action.sender = Some(sender);
        attack_action
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
