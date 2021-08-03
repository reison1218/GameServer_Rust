use crate::robot::robot_status::RobotStatus;
use crate::robot::robot_task_mgr::RobotTask;
use crate::robot::RobotActionType;
use crate::JsonValue;
use crossbeam::channel::Sender;
use log::error;
use serde_json::Map;
use tools::cmd_code::BattleCode;

///机器人状态行为trait
pub trait RobotStatusAction {
    fn set_sender(&mut self, sender: Sender<RobotTask>);

    fn enter(&self);
    fn execute(&self);
    fn exit(&self);
    fn get_status(&self) -> RobotStatus;
    fn get_robot_id(&self) -> u32;
    fn get_sender(&self) -> &Sender<RobotTask>;
    fn send_2_battle(
        &self,
        target_index: usize,
        robot_action_type: RobotActionType,
        cmd: BattleCode,
    ) {
        let mut robot_task = RobotTask::default();
        robot_task.action_type = robot_action_type;
        robot_task.robot_id = self.get_robot_id();
        let mut map = Map::new();
        map.insert("target_index".to_owned(), JsonValue::from(target_index));
        map.insert("cmd".to_owned(), JsonValue::from(cmd.into_u32()));
        robot_task.data = JsonValue::from(map);
        let res = self.get_sender().send(robot_task);
        if let Err(e) = res {
            error!("{:?}", e);
        }
    }
}
