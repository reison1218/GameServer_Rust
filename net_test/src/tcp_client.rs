use tools::protos::protocol::{S_USER_LOGIN, C_USER_LOGIN, C_SYNC_DATA, S_SYNC_DATA};
use std::time::Duration;
use tools::util::packet::Packet;
use std::io::Write;
use protobuf::Message;
use std::net::TcpStream;
use tools::tcp::ClientHandler;
use tools::cmd_code::{GameCode, RoomCode, ClientCode};
use futures::executor::block_on;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use crate::ID;
use futures::AsyncWriteExt;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicU32;
use tools::protos::room::S_ROOM;
use tools::protos::base::PlayerPt;

pub fn test_tcp_client(pid:&str){
        let uid = block_on(crate::test_http_client(pid));
        let uid = uid.unwrap();
        let mut tcp_client = TcpClientHandler::new();
        //tcp_client.on_read("192.168.1.100:16801".to_string());
        tcp_client.on_read("localhost:16801".to_string());
}

pub fn test_tcp_clients(){
    for i in 1..=2000{
        let sr = i.to_string();
        let uid = block_on(crate::test_http_client(sr.as_str()));
        let uid = uid.unwrap();

        let m = move || {
            let mut tcp_client = TcpClientHandler::new();
            //tcp_client.on_read("192.168.1.100:16801".to_string());
            tcp_client.on_read("localhost:16801".to_string());
        };
        std::thread::spawn(m);
        std::thread::sleep(Duration::from_millis(100));
        println!("client:{}",i);
    }

    // let mut tcp_client = TcpClientHandler::new();
    // tcp_client.on_read("127.0.0.1:16801".to_string());
}
pub struct TcpClientHandler {
    ts: Option<TcpStream>,
}

impl TcpClientHandler {
    pub fn new() -> TcpClientHandler {
        let mut tch = TcpClientHandler { ts: None};
        tch
    }
}

impl ClientHandler for TcpClientHandler {
    fn on_open(&mut self, ts: TcpStream) {
        self.ts = Some(ts);
        let mut packet = Packet::default();
        packet.set_cmd(GameCode::Login as u32);

        let mut s_l = tools::protos::protocol::C_USER_LOGIN::new();
        let mut write:RwLockWriteGuard<AtomicU32> = ID.write().unwrap();
        write.fetch_add(1, Ordering::Relaxed);
        let id = write.load(Ordering::Relaxed);
        s_l.set_user_id(id);
        packet.set_data(&s_l.write_to_bytes().unwrap()[..]);
        packet.set_len(16+packet.get_data().len() as u32);
        self.ts.as_mut().unwrap().write(&packet.build_client_bytes()[..]).unwrap();
        self.ts.as_mut().unwrap().flush().unwrap();

        std::thread::sleep(Duration::from_secs(2));
        let mut c_r = C_SYNC_DATA::new();
        let mut pp = PlayerPt::new();
        let mut v = Vec::new();
        v.push(1);
        v.push(2);
        pp.dlc = v;
        pp.nick_name="test111".to_string();
        c_r.set_player_pt(pp);
        packet.set_cmd(GameCode::SyncData as u32);
        packet.set_data(&c_r.write_to_bytes().unwrap()[..]);
        packet.set_len(16+packet.get_data().len() as u32);
        self.ts.as_mut().unwrap().write(&packet.build_client_bytes()[..]).unwrap();
        self.ts.as_mut().unwrap().flush().unwrap();
    }

    fn on_close(&mut self) {
        println!("断开链接");
        //let address = "192.168.1.100:16801";
        let address = "localhost:16801";
        self.on_read(address.to_string());
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        let packet = Packet::from_only_client(mess).unwrap();
        if packet.get_cmd() == ClientCode::Login as u32{
            let mut s = S_USER_LOGIN::new();
            s.merge_from_bytes(packet.get_data());
            println!("from server-login:{:?}----{:?}",packet,s);
        }else if packet.get_cmd() == ClientCode::Room as u32{
            let mut s = S_ROOM::new();
            s.merge_from_bytes(packet.get_data());
            println!("from server-room:{:?}----{:?}",packet,s);
        }else if packet.get_cmd() == ClientCode::SyncData as u32{
            let mut s = S_SYNC_DATA::new();
            s.merge_from_bytes(packet.get_data());
            println!("from server-sync:{:?}----{:?}",packet,s);
        }

    }

    fn get_address(&self) -> &str {
        //let address = "192.168.1.100:16801";
        let address = "localhost:16801";
        address
    }
}