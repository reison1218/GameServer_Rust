use crate::mgr::game_center_mgr::GameCenterMgr;
use crate::net::Forward;
use async_std::sync::Mutex;
use async_std::task::block_on;
use async_trait::async_trait;
use log::{error, info};
use std::sync::Arc;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

#[derive(Clone)]
pub struct BattleTcpServerHandler {
    pub gm: Arc<Mutex<GameCenterMgr>>,
}

unsafe impl Send for BattleTcpServerHandler {}

unsafe impl Sync for BattleTcpServerHandler {}

impl Forward for BattleTcpServerHandler {
    fn get_game_center_mut(&mut self) -> &mut Arc<Mutex<GameCenterMgr>> {
        &mut self.gm
    }
}

#[async_trait]
impl tools::tcp::Handler for BattleTcpServerHandler {
    async fn try_clone(&self) -> Self {
        self.clone()
    }

    ///客户端tcp链接激活事件
    async fn on_open(&mut self, sender: TcpSender) {
        self.gm.lock().await.add_battle_client(sender);
    }

    ///客户端tcp链接关闭事件
    async fn on_close(&mut self) {
        info!("Battle-Listener与tcp客户端断开连接！");
    }

    ///客户端读取事件
    async fn on_message(&mut self, mess: Vec<u8>) {
        let packet_array = Packet::build_array_from_server(mess);

        if let Err(e) = packet_array {
            error!("{:?}", e);
            return;
        }
        let packet_array = packet_array.unwrap();

        self.forward_packet(packet_array).await;
    }
}

///创建新的tcp服务器,如果有问题，终端进程
pub fn new(address: String, rm: Arc<Mutex<GameCenterMgr>>) {
    let sh = BattleTcpServerHandler { gm: rm };
    let m = async move {
        let _ = block_on(tools::tcp::tcp_server::new(address, sh));
    };
    async_std::task::spawn(m);
}
