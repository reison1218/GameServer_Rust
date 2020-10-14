use crate::fsm::miner::Miner;
use std::collections::HashMap;
use tools::tcp::TcpSender;

///channel管理结构体
#[derive(Default)]
pub struct RobotMgr {
    sender: Option<TcpSender>,                    //tcp channel的发送方
    robot_map: HashMap<u64, HashMap<u32, Miner>>, //机器人map
}

impl RobotMgr {
    pub fn set_sender(&mut self, sender: TcpSender) {
        self.sender = Some(sender);
    }
}
