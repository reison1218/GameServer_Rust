use crate::goal_ai::cter::Cter;
use std::collections::HashMap;
use tools::tcp::TcpSender;

///channel管理结构体
#[derive(Default)]
pub struct RobotMgr {
    pub sender: Option<TcpSender>,     //tcp channel的发送方
    pub robot_map: HashMap<u32, Cter>, //机器人map
}

impl RobotMgr {
    pub fn set_sender(&mut self, sender: TcpSender) {
        self.sender = Some(sender);
    }
}
