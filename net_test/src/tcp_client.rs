use tools::protos::protocol::{S_USER_LOGIN, C_USER_LOGIN, C_SYNC_DATA, S_SYNC_DATA};
use std::time::Duration;
use tools::util::packet::Packet;
use std::io::Write;
use protobuf::Message;
use std::net::TcpStream;
use tools::tcp::ClientHandler;
use tools::cmd_code::{GameCode, RoomCode, ClientCode};
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use crate::ID;
use async_trait::async_trait;

use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicU32;
use tools::protos::room::{S_ROOM, C_CREATE_ROOM, C_SEARCH_ROOM, C_JOIN_ROOM, C_PREPARE_CANCEL, S_PREPARE_CANCEL, C_CHOOSE_CHARACTER, S_CHOOSE_CHARACTER, S_ROOM_ADD_MEMBER_NOTICE};
use tools::protos::base::{PlayerPt, CharacterPt};
use tools::http::send_http_request;
use serde_json::Value;
use std::str::FromStr;
use async_std::task::block_on;

pub fn test_tcp_client(pid:&str){
        let uid = block_on(crate::test_http_client(pid));
        if uid.is_err(){
            println!("{:?}",uid.err().unwrap().to_string());
            return;
        }
        let mut tcp_client = TcpClientHandler::new();
        tcp_client.user_id = uid.unwrap();
        //tcp_client.on_read("192.168.1.100:16801".to_string());
        block_on(tcp_client.on_read("localhost:16801".to_string()));
}

pub fn test_tcp_clients(){
    for i in 1..=3000{
        let sr = i.to_string();
        let uid = async_std::task::block_on(crate::test_http_client(sr.as_str()));
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

    let mut tcp_client = TcpClientHandler::new();
    tcp_client.on_read("127.0.0.1:16801".to_string());
    //tcp_client.on_read("192.168.1.100:16801".to_string());
}
pub struct TcpClientHandler {
    ts: Option<TcpStream>,
    pub user_id:u32
}

impl TcpClientHandler {
    pub fn new() -> TcpClientHandler {
        let tch = TcpClientHandler { ts: None,user_id:0 as u32};
        tch
    }
}

#[async_trait]
impl ClientHandler for TcpClientHandler {
    async fn on_open(&mut self, ts: TcpStream) {
        self.ts = Some(ts);
        let mut s_l = tools::protos::protocol::C_USER_LOGIN::new();
        let mut packet = Packet::default();
        packet.set_cmd(GameCode::Login as u32);
        let mut write:RwLockWriteGuard<AtomicU32> = ID.write().unwrap();
        write.fetch_add(1, Ordering::Relaxed);
        let id = write.load(Ordering::Relaxed);
        std::mem::drop(write);
        // let id = id.to_string();
        //
        // let str = r#"{"platform_id":"test","game_id":101,"register_platform":"test","nick_name":"test","phone_no":"123131231"}"#;
        // let mut value = Value::from_str(str).unwrap();
        // let map = value.as_object_mut().unwrap();
        // map.insert("platform_id".to_string(),Value::from(id.to_string()));
        // map.insert("test".to_string(),Value::from(id.to_string()));
        // let res = send_http_request("192.168.1.100:8888","/center/user_id","post",Some(value));
        // let res = block_on(res);
        // let mut res = res.unwrap();
        // let res_map = res.as_object_mut().unwrap();
        // let user_id = res_map.get("user_id").unwrap();
        // let user_id = user_id.as_u64().unwrap() as u32;
        //s_l.set_user_id(id);
        s_l.set_user_id(self.user_id);
        packet.set_data(&s_l.write_to_bytes().unwrap()[..]);
        packet.set_len(16+packet.get_data().len() as u32);
        self.ts.as_mut().unwrap().write(&packet.build_client_bytes()[..]).unwrap();
        self.ts.as_mut().unwrap().flush().unwrap();

        //panic!("");
        std::thread::sleep(Duration::from_secs(2));

        let mut  csr = C_SEARCH_ROOM::new();
        csr.set_room_type(2 as u32);
        let bytes = Packet::build_packet_bytes(GameCode::SearchRoom as u32,self.user_id,csr.write_to_bytes().unwrap(),false,true);
        self.ts.as_mut().unwrap().write(&bytes[..]).unwrap();
        self.ts.as_mut().unwrap().flush().unwrap();
        std::thread::sleep(Duration::from_secs(2));

        let mut ccc = C_CHOOSE_CHARACTER::new();
        let mut write = ID.write().unwrap();
        write.fetch_add(1,Ordering::SeqCst);
        let cter_id = write.load(Ordering::SeqCst);
        let mut v = Vec::new();
        v.push(1001_u32);
        v.push(1002_u32);
        // ccc.set_cter_id(cter_id);
        ccc.set_cter_id(1001);
        packet.set_cmd(RoomCode::ChoiceCharacter as u32);
        packet.set_data(&ccc.write_to_bytes().unwrap()[..]);
        packet.set_len(16+packet.get_data().len() as u32);
        self.ts.as_mut().unwrap().write(&packet.build_client_bytes()[..]).unwrap();
        self.ts.as_mut().unwrap().flush().unwrap();
        //std::thread::sleep(Duration::from_secs(2));

        // let mut cpc = C_PREPARE_CANCEL::new();
        // cpc.prepare = true;
        // packet.set_cmd(RoomCode::PrepareCancel as u32);
        // packet.set_data(&cpc.write_to_bytes().unwrap()[..]);
        // packet.set_len(16+packet.get_data().len() as u32);
        // self.ts.as_mut().unwrap().write(&packet.build_client_bytes()[..]).unwrap();
        // self.ts.as_mut().unwrap().flush().unwrap();

        // std::thread::sleep(Duration::from_secs(2));
        //
        // let mut cjr = C_JOIN_ROOM::new();
        // cjr.room_id = 458301785;
        // cjr.room_type = 1_u32;
        // let bytes = Packet::build_packet_bytes(GameCode::JoinRoom as u32,self.user_id,cjr.write_to_bytes().unwrap(),false,true);
        // self.ts.as_mut().unwrap().write(&bytes[..]).unwrap();
        // self.ts.as_mut().unwrap().flush().unwrap();

        // let mut c_r = C_CREATE_ROOM::new();
        // c_r.room_type = 1 as u32;
        // packet.set_cmd(GameCode::CreateRoom as u32);
        // packet.set_data(&c_r.write_to_bytes().unwrap()[..]);
        // packet.set_len(16+packet.get_data().len() as u32);
        // self.ts.as_mut().unwrap().write(&packet.build_client_bytes()[..]).unwrap();
        // self.ts.as_mut().unwrap().flush().unwrap();




        // let mut c_r = C_SYNC_DATA::new();
        // let mut pp = PlayerPt::new();
        // let mut v = Vec::new();
        // v.push(1);
        // v.push(2);
        // pp.dlc = v;
        // pp.nick_name="test111".to_string();
        // c_r.set_player_pt(pp);
        // packet.set_cmd(GameCode::SyncData as u32);
        // packet.set_data(&c_r.write_to_bytes().unwrap()[..]);
        // packet.set_len(16+packet.get_data().len() as u32);
        // self.ts.as_mut().unwrap().write(&packet.build_client_bytes()[..]).unwrap();
        // self.ts.as_mut().unwrap().flush().unwrap();
    }

    async fn on_close(&mut self) {
        println!("断开链接");
        //let address = "192.168.1.100:16801";
        let address = "localhost:16801";
        self.on_read(address.to_string()).await;
    }

    async fn on_message(&mut self, mess: Vec<u8>) {
        let packet = Packet::from_only_client(mess.clone()).unwrap();
        if packet.get_cmd() == ClientCode::Login as u32{
            let mut s = S_USER_LOGIN::new();
            s.merge_from_bytes(packet.get_data());
            println!("from server-login:{:?}",s);
        }else if packet.get_cmd() == ClientCode::Room as u32{
            let mut s = S_ROOM::new();
            s.merge_from_bytes(packet.get_data());
            println!("from server-room:{:?}",s);
        }else if packet.get_cmd() == ClientCode::SyncData as u32{
            let mut s = S_SYNC_DATA::new();
            s.merge_from_bytes(packet.get_data());
            println!("from server-sync:{:?}",s);
        }else if packet.get_cmd() == ClientCode::PrepareCancel as u32{
            let mut s = S_PREPARE_CANCEL::new();
            s.merge_from_bytes(packet.get_data());
            println!("from server-prepare:{:?}",s);
        }else if packet.get_cmd() == ClientCode::ChoiceCharacter as u32{
            let mut s = S_CHOOSE_CHARACTER::new();
            s.merge_from_bytes(packet.get_data());
            println!("from server-choosecter:{:?}",s);
        }else if packet.get_cmd() == ClientCode::RoomAddMemberNotice as u32{
            let mut s = S_ROOM_ADD_MEMBER_NOTICE::new();
            s.merge_from_bytes(packet.get_data());
            println!("client:{},from server-mem-notice:{:?}",self.user_id,s);
        }
    }
}