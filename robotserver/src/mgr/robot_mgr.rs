use crate::goal_ai::cter::Cter;
use crate::handlers::robot_handler::request_robot;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use tools::cmd_code::RobotCode;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

///channel管理结构体
#[derive(Default)]
pub struct RobotMgr {
    pub robot_map: HashMap<u64, HashMap<u32, Cter>>, //机器人map
    pub cmd_map: HashMap<u32, fn(&mut RobotMgr, Packet) -> anyhow::Result<()>, RandomState>, //命令管理 key:cmd,value:函数指针
    pub sender: Option<TcpSender>, //tcp channel的发送方
}

tools::get_mut_ref!(RobotMgr);

impl RobotMgr {
    pub fn new() -> Self {
        let mut rm = RobotMgr::new();
        rm.cmd_map
            .insert(RobotCode::RequestRobot.into_u32(), request_robot);
        rm
    }

    pub fn set_sender(&mut self, sender: TcpSender) {
        self.sender = Some(sender);
    }
}
