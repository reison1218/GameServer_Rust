use crate::Lock;
use async_std::task::{block_on, spawn};
use async_trait::async_trait;
use log::{error, warn};
use tools::tcp_message_io::{MessageHandler, TcpHandler, TransportWay};
use tools::util::packet::Packet;

///处理客户端所有请求,每个客户端单独分配一个handler
#[derive(Clone)]
pub struct TcpClientHandler {
    pub bm: Lock,
}

impl TcpClientHandler {
    pub fn new(bm: Lock) -> Self {
        let tch = TcpClientHandler { bm };
        tch
    }
}

#[async_trait]
impl MessageHandler for TcpClientHandler {
    async fn on_open(&mut self, tcp_handler: TcpHandler) {
        let mut lock = self.bm.lock().await;
        lock.set_game_center_channel(tcp_handler);
    }

    async fn on_close(&mut self) {
        let address = crate::CONF_MAP.get_str("tcp_port");
        self.connect(TransportWay::Tcp, address).await;
    }

    async fn on_message(&mut self, mess: &[u8]) {
        let packet_array = Packet::build_array_from_server(mess.to_vec());

        if let Err(e) = packet_array {
            error!("{:?}", e.to_string());
            return;
        }
        let packet_array = packet_array.unwrap();
        //遍历命令，并执行
        for packet in packet_array {
            let cmd = packet.get_cmd();
            if cmd <= 0 {
                warn!("cmd is invalid!cmd = {}", cmd);
                continue;
            }
            spawn(handler_mess_s(self.bm.clone(), packet));
        }
    }

    async fn try_clone(&self) -> Self {
        self.clone()
    }
}

async fn handler_mess_s(bm: Lock, packet: Packet) {
    let mut lock = bm.lock().await;
    lock.invok(packet);
}

///创建新的tcp服务器,如果有问题，终端进程
pub fn new(address: &str, bm: Lock) {
    let mut tch = TcpClientHandler::new(bm);
    let res = tch.connect(TransportWay::Tcp, address);
    block_on(res);
}
