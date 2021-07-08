use crate::net::Forward;
use crate::Lock;
use async_std::task::block_on;
use async_trait::async_trait;
use log::error;
use tools::tcp_message_io::{MessageHandler, TcpHandler, TransportWay};
use tools::util::packet::Packet;

///处理客户端所有请求,每个客户端单独分配一个handler
#[derive(Clone)]
pub struct RoomTcpClientHandler {
    pub gm: Lock,
}

impl Forward for RoomTcpClientHandler {
    fn get_battle_token(&self) -> Option<usize> {
        None
    }

    fn get_gate_token(&self) -> Option<usize> {
        None
    }

    fn get_game_center_mut(&mut self) -> &mut Lock {
        &mut self.gm
    }
}

#[async_trait]
impl MessageHandler for RoomTcpClientHandler {
    async fn on_open(&mut self, ts: TcpHandler) {
        let mut lock = block_on(self.gm.lock());
        lock.set_room_sender(ts);
    }

    async fn on_close(&mut self) {
        let address = crate::CONF_MAP.get_str("room_port");

        self.connect(TransportWay::Tcp, address).await;
    }

    async fn on_message(&mut self, mess: &[u8]) {
        let packet_array = Packet::build_array_from_server(mess.to_vec());

        if let Err(e) = packet_array {
            error!("{:?}", e.to_string());
            return;
        }
        let packet_array = packet_array.unwrap();
        //转发消息
        self.forward_packet(packet_array).await;
    }

    async fn try_clone(&self) -> Self {
        self.clone()
    }
}
