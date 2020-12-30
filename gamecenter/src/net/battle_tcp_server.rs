use crate::mgr::game_center_mgr::GameCenterMgr;
use crate::net::handler_mess_s;
use async_std::sync::Mutex;
use async_std::task;
use async_std::task::block_on;
use async_trait::async_trait;
use log::{error, info};
use std::sync::Arc;
use tools::cmd_code::RoomCode;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

#[derive(Clone)]
pub struct BattleTcpServerHandler {
    pub token: usize,
    pub gm: Arc<Mutex<GameCenterMgr>>,
}

unsafe impl Send for BattleTcpServerHandler {}

unsafe impl Sync for BattleTcpServerHandler {}

#[async_trait]
impl tools::tcp::Handler for BattleTcpServerHandler {
    async fn try_clone(&self) -> Self {
        self.clone()
    }

    ///客户端tcp链接激活事件
    async fn on_open(&mut self, sender: TcpSender) {
        self.token = sender.token;
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

        for packet in packet_array {
            //判断是否是房间服的命令，如果不是，则直接无视掉
            if packet.get_cmd() < RoomCode::Min.into_u32()
                || packet.get_cmd() > RoomCode::Max.into_u32()
            {
                error!("the cmd:{} is invalid!", packet.get_cmd());
                continue;
            }
            //异步处理业务逻辑
            task::spawn(handler_mess_s(self.gm.clone(), packet));
        }
    }
}

///创建新的tcp服务器,如果有问题，终端进程
pub fn new(address: &str, rm: Arc<Mutex<GameCenterMgr>>) {
    let sh = BattleTcpServerHandler { gm: rm };
    let res = block_on(tools::tcp::tcp_server::new(address, sh));
    if let Err(e) = res {
        error!("{:?}", e);
        std::process::abort();
    }
}
