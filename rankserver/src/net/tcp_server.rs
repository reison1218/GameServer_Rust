use crate::mgr::rank_mgr::RankMgr;
use async_std::sync::Mutex;
use async_std::task;
use async_std::task::block_on;
use async_trait::async_trait;
use log::{error, info};
use std::sync::Arc;
use tools::tcp::{tcp_server, TcpSender};
use tools::util::packet::Packet;
///处理客户端所有请求,每个客户端单独分配一个handler
#[derive(Clone)]
pub struct TcpServerHandler {
    pub rm: Arc<Mutex<RankMgr>>,
}

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

#[async_trait]
impl tools::tcp::Handler for TcpServerHandler {
    async fn try_clone(&self) -> Self {
        self.clone()
    }

    ///客户端tcp链接激活事件
    async fn on_open(&mut self, sender: TcpSender) {
        self.rm.lock().await.set_sender(sender);
    }

    ///客户端tcp链接关闭事件
    async fn on_close(&mut self) {
        info!("与tcp客户端断开连接");
    }

    ///客户端读取事件
    async fn on_message(&mut self, mess: Vec<u8>) {
        let packet_array = Packet::build_array_from_server(mess);

        if let Err(e) = packet_array {
            error!("{:?}", e);
            return;
        }
        let packet_array = packet_array.unwrap();

        for packet in packet_array {
            //异步处理业务逻辑
            //task::spawn(handler_mess_s(self.rm.clone(), packet));
        }
    }
}

// ///处理客户端消息
// async fn handler_mess_s(rm: Arc<Mutex<RankMgr>>, packet: Packet) {
//     let mut lock = rm.lock().await;
//     lock.invok(packet);
// }

///创建新的tcp服务器,如果有问题，终端进程
pub fn new(address: &str, rm: Arc<Mutex<RankMgr>>) {
    let sh = TcpServerHandler { rm };
    let res = block_on(tcp_server::new(address.to_string(), sh));
    if let Err(e) = res {
        error!("{:?}", e);
        std::process::abort();
    }
}
