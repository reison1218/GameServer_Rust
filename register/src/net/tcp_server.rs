use super::*;
use crate::ID;
use protobuf::ProtobufEnum;
use std::sync::atomic::Ordering;
use tools::tcp::TcpSender;
use std::sync::{RwLock, Arc};
use crate::mgr::register_mgr::RegisterMgr;


struct TcpServerHandler {
    pub tcp: Option<TcpSender>, //相当于channel
    pub add: Option<String>,     //客户端地址
    cm: Arc<RwLock<RegisterMgr>>, //channel管理器
}

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

impl tools::tcp::Handler for TcpServerHandler {
    fn try_clone(&self) -> Self {
        let mut tcp: Option<TcpSender> = None;
        if self.tcp.is_some() {
            tcp = Some(self.tcp.as_ref().unwrap().clone());
        }

        TcpServerHandler {
            tcp: tcp,
            add: self.add.clone(),
            cm: self.cm.clone(),
        }
    }

    fn on_open(&mut self, sender: TcpSender) {
        self.tcp = Some(sender);
    }

    fn on_close(&mut self) {
        info!(
            "tcp_server:客户端断开连接,通知其他服卸载玩家数据:{}",
            self.add.as_ref().unwrap()
        );
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        info!("GateServer got message '{:?}'. ", mess);
    }
}

impl TcpServerHandler {
    ///处理二进制数据
    fn handle_binary(&mut self, mut mess: MessPacketPt) {

    }

    ///数据包转发
    fn arrange_packet(&mut self, mess: MessPacketPt) {
    }
}

pub fn new(address: &str, cm: Arc<RwLock<ChannelMgr>>) {
    let sh = TcpServerHandler {
        tcp: None,
        cm: cm,
        add: Some(address.to_string()),
    };
    tools::tcp::tcp_server::new(address,sh);
}
