use crate::mgr::robot_mgr::RobotMgr;
use log::info;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use tools::tcp::{ClientHandler, TcpSender};

///处理客户端所有请求,每个客户端单独分配一个handler
pub struct TcpServerHandler {
    pub sender: Option<TcpSender>,
    pub rm: Arc<Mutex<RobotMgr>>,
}

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

impl tools::tcp::Handler for TcpServerHandler {
    fn try_clone(&self) -> Self {
        let mut sender: Option<TcpSender> = None;
        if self.sender.is_some() {
            sender = Some(self.sender.as_ref().unwrap().clone());
        }
        TcpServerHandler {
            sender,
            rm: self.rm.clone(),
        }
    }

    fn on_open(&mut self, sender: TcpSender) {
        self.rm.lock().unwrap().set_sender(sender);
    }

    fn on_close(&mut self) {
        info!("与tcp客户端断开连接");
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        //todo
    }
}
