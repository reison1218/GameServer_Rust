use crate::robot::robot_status::robot_status::RobotStatus;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

///机器人状态行为trait
pub trait RobotStatusAction {
    fn set_sender(&self, sender: Sender<RobotTask>);
    fn get_cter_id(&self) -> u32;
    fn enter(&self);
    fn execute(&self);
    fn exit(&self);
    fn get_status(&self) -> RobotStatus;
}
