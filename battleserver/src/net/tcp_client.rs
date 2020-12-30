use crate::mgr::battle_mgr::BattleMgr;
use async_std::sync::{Arc, Mutex};
use async_std::task::block_on;
use async_trait::async_trait;
use log::{error, warn};
use std::net::TcpStream;
use tools::tcp::ClientHandler;
use tools::util::packet::Packet;

///处理客户端所有请求,每个客户端单独分配一个handler
#[derive(Clone)]
pub struct TcpClientHandler {
    pub bm: Arc<Mutex<BattleMgr>>,
}

#[async_trait]
impl ClientHandler for TcpClientHandler {
    async fn on_open(&mut self, ts: TcpStream) {
        let mut lock = block_on(self.bm.lock());
        lock.set_game_center_channel(ts);
    }

    async fn on_close(&mut self) {
        let address = crate::CONF_MAP.get_str("game_center_port");

        self.on_read(address.to_string()).await;
    }

    async fn on_message(&mut self, mess: Vec<u8>) {
        let packet_array = Packet::build_array_from_server(mess);

        if let Err(e) = packet_array {
            error!("{:?}", e.to_string());
            return;
        }
        let packet_array = packet_array.unwrap();
        //遍历命令，并执行
        for mut packet in packet_array {
            let cmd = packet.get_cmd();
            //判断是否是发给客户端消息
            if packet.get_cmd() > 0 {
                let mut lock = block_on(self.bm.lock());
                let f = lock.cmd_map.get_mut(&cmd);
                match f {
                    Some(f) => {
                        let res = f(&mut lock, packet.clone());
                        if let Err(e) = res {
                            warn!("{:?}", e);
                        }
                    }
                    None => {
                        warn!("could not find function！cmd:{}", cmd);
                    }
                }
            } else {
                //todo 暂时不做处理
            }
        }
    }
}
