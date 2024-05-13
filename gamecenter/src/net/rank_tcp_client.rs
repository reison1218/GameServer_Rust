use crate::net::Forward;
use crate::Lock;
use async_std::task::block_on;
use async_trait::async_trait;
use crossbeam::channel::Sender;
use log::error;
use tools::tcp::ClientHandler;
use tools::util::packet::Packet;

///处理排行榜服所有请求,每个客户端单独分配一个handler
#[derive(Clone)]
pub struct RankTcpClientHandler {
    pub gm: Lock,
}

impl Forward for RankTcpClientHandler {
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
impl ClientHandler for RankTcpClientHandler {
    async fn on_open(&mut self, ts: Sender<Vec<u8>>) {
        let mut lock = block_on(self.gm.lock());
        lock.set_rank_sender(ts);
    }

    async fn on_close(&mut self) {
        let address = crate::CONF_MAP.get_str("rank_port", "");

        self.on_read(address.to_string()).await;
    }

    async fn on_message(&mut self, mess: Vec<u8>) {
        let packet_array = Packet::build_array_from_server(mess);

        if let Err(e) = packet_array {
            error!("{:?}", e.to_string());
            return;
        }
        let packet_array = packet_array.unwrap();
        //转发消息
        self.forward_packet(packet_array).await;
    }
}
