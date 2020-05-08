use super::*;
use protobuf::{ProtobufEnum, Message};
use std::sync::atomic::Ordering;
use tools::tcp::TcpSender;
use std::sync::{RwLock, Arc};
use crate::mgr::register_mgr::RegisterMgr;
use crate::entity::NetClient::{NetClient, ClientType};
use tools::protos::base::MessPacketPt;
use crate::GATE_ID;
use crate::ROOM_ID;
use crate::entity::NetClient::ClientType::*;
use crate::mgr::id_contants::*;

struct TcpServerHandler {
    pub tcp: Option<TcpSender>, //相当于channel
    pub add: Option<String>,     //客户端地址
    rm: Arc<RwLock<RegisterMgr>>, //channel管理器
}

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

impl tools::tcp::Handler for TcpServerHandler {
    fn try_clone(&self) -> Self {
        let mut tcp: Option<TcpSender> = None;
        if self.tcp.is_some() {
            tcp = Some(self.tcp.as_ref().unwrap().clone());
        }

        TcpServerHandler {
            tcp,
            add: self.add.clone(),
            rm: self.rm.clone(),
        }
    }

    fn on_open(&mut self, sender: TcpSender) {
        self.tcp = Some(sender);
    }

    fn on_close(&mut self) {
        info!(
            "tcp_server:客户端断开连接,通知其他服卸载玩家数据:{}",
            self.add.as_ref().unwrap()
        );
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        let mut mp = MessPacketPt::new();
        mp.merge_from_bytes(&mess[..]);
        self.handle_binary(mp);

    }
}

impl TcpServerHandler {
    ///处理二进制数据
    fn handle_binary(&mut self, mut mess: MessPacketPt) {
        //如果是客户端消息，直接返回
        if mess.is_client{
            return;
        }
        let regist = true;
        //如果是注册
        if regist{
            //封装gate和room客户端
            let mut id:Option<u32> = None;
            let mut client_type:Option<ClientType> = None;
            if mess.is_broad{
                let mut gate_id = GATE_ID.write().unwrap();
                gate_id.store(1, Ordering::Relaxed);
                id = Some(gate_id.load(Ordering::Relaxed));
                client_type = Some(ClientType::GateServer);
            }else{
                let mut room_id = ROOM_ID.write().unwrap();
                room_id.store(1, Ordering::Relaxed);
                id = Some(room_id.load(Ordering::Relaxed));
                client_type = Some(ClientType::RoomServer);
            }
            let mut write = self.rm.write().unwrap();
            let net_client = NetClient::new(ClientType::GateServer as u8,id.unwrap(),"tst".to_string(),self.tcp.clone());
            match client_type.unwrap() {
                GateServer=>{
                    write.gate_channel.insert(net_client.get_id(),net_client);
                },
                RoomServer=>{
                    write.room_channel.insert(net_client.get_id(),net_client);
                }
            }

        }else{//一般消息
            let id = 0;
            //gate消息
            if id<=GateId::Max as u32 && id>=GateId::Min as u32{
                //判断是否有绑定
                let write = self.rm.write().unwrap();
                if !write.g2r.contains_key(&id){
                   if write.room_channel.len()<=0{
                       return;
                   }
                }

            }else if id<=RoomId::Max as u32 && id>=RoomId::Min as u32{//room消息

            }
        }

    }

    ///数据包转发
    fn arrange_packet(&mut self, mess: MessPacketPt) {
    }
}

pub fn new(address: &str, rm: Arc<RwLock<RegisterMgr>>) {
    let sh = TcpServerHandler {
        tcp: None,
        rm: rm,
        add: Some(address.to_string()),
    };
    tools::tcp::tcp_server::new(address,sh);
}
