use crate::ID;
use async_trait::async_trait;
use crossbeam::channel::Sender;
use protobuf::Message;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use std::time::Duration;
use tools::cmd_code::{ClientCode, GameCode, RoomCode};
use tools::protos::protocol::{C_SYNC_DATA, C_USER_LOGIN, S_SYNC_DATA, S_USER_LOGIN};
use tools::tcp::ClientHandler;
use tools::util::packet::Packet;

use async_std::task::block_on;
use serde_json::Value;
use std::str::FromStr;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use tools::protos::base::{CharacterPt, PlayerPt};
use tools::protos::room::{
    C_CHOOSE_CHARACTER, C_CREATE_ROOM, C_JOIN_ROOM, C_PREPARE_CANCEL, C_SEARCH_ROOM,
    S_CHOOSE_CHARACTER, S_PREPARE_CANCEL, S_ROOM, S_ROOM_ADD_MEMBER_NOTICE,
};

pub fn test_tcp_client(pid: &str) {
    let uid = block_on(crate::test_http_client(pid));
    if uid.is_err() {
        println!("{:?}", uid.err().unwrap().to_string());
        return;
    }
    let mut tcp_client = TcpClientHandler::new(pid.to_owned());
    tcp_client.user_id = uid.unwrap();
    // block_on(tcp_client.on_read("192.168.1.100:16801".to_string()));
    block_on(tcp_client.on_read("localhost:16801".to_string()));
}

pub struct TcpClientHandler {
    ts: Option<Sender<Vec<u8>>>,
    pub platform_value: String,
    pub user_id: u32,
}

impl TcpClientHandler {
    pub fn new(platform_value: String) -> TcpClientHandler {
        let tch = TcpClientHandler {
            ts: None,
            platform_value: platform_value,
            user_id: 0 as u32,
        };
        tch
    }
}

#[async_trait]
impl ClientHandler for TcpClientHandler {
    async fn on_open(&mut self, ts: Sender<Vec<u8>>) {
        self.ts = Some(ts);
        let mut s_l = tools::protos::protocol::C_USER_LOGIN::new();

        s_l.set_user_id(self.user_id);
        s_l.set_register_platform("test".to_owned());
        s_l.set_platform_value(self.platform_value.clone());
        let mut packet = Packet::default();
        packet.set_cmd(GameCode::Login as u32);
        packet.set_data(&s_l.write_to_bytes().unwrap()[..]);
        packet.set_len(16 + packet.get_data().len() as u32);
        self.ts
            .as_mut()
            .unwrap()
            .send(packet.build_client_bytes())
            .unwrap();
    }

    async fn on_close(&mut self) {
        println!("断开链接");
        //let address = "192.168.1.100:16801";
        let address = "localhost:16801";
        self.on_read(address.to_string()).await;
    }

    async fn on_message(&mut self, mess: Vec<u8>) {
        let packet = Packet::from_only_client(mess.clone()).unwrap();
        if packet.get_cmd() == ClientCode::Login as u32 {
            let mut s = S_USER_LOGIN::new();
            s.merge_from_bytes(packet.get_data());
            println!("from server-login:{:?}", s);
        }
    }
}
