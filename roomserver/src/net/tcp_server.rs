use super::*;
use std::sync::{Arc, RwLock};
use crate::entity::room::Room;
use crate::mgr::room_mgr::RoomMgr;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use protobuf::{Message, ProtobufEnum};
use tools::cmd_code::RoomCode;
use std::path::Component::RootDir;
use tools::util::packet::Packet;
use tools::tcp::{TcpSender,Data};
use tools::tcp::tcp_server;
use tools::protos::base::MessPacketPt;

pub struct TcpServerHandler{
    pub sender:Option<TcpSender>,
    pub rm: Arc<RwLock<RoomMgr>>,

}

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

impl tools::tcp::Handler for TcpServerHandler{
    fn try_clone(&self) -> Self {
        let mut sender: Option<TcpSender> = None;
        if self.sender.is_some() {
            sender = Some(self.sender.as_ref().unwrap().clone());
        }
        TcpServerHandler {
           sender,
            rm: self.rm.clone(),
        }
    }

    fn on_open(&mut self, sender: TcpSender) {
        self.rm.write().unwrap().sender = Some(sender);
    }

    fn on_close(&mut self) {
        info!("与tcp客户端断开连接");
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        let mut mp = MessPacketPt::new();
        mp.merge_from_bytes(&mess[..]);
        //判断是否是房间服的命令，如果不是，则直接无视掉
        if mp.get_cmd() < RoomCode::Min as u32 || mp.get_cmd()>RoomCode::Max as u32{
            error!("the cmd:{} is not belong roomserver!",mp.get_cmd());
            return;
        }
        if mp.get_data().is_empty() && mp.get_cmd()!=RoomCode::LineOff as u32{
            error!("the cmd:{}'s mess's data is null!",mp.get_cmd());
            return;
        }
        //异步处理业务逻辑
        async_std::task::spawn(handler_mess_s(self.rm.clone(),mp));
    }
}

async fn handler_mess_s(rm: Arc<RwLock<RoomMgr>>, mut mess: MessPacketPt) {
    let mut write = rm.write().unwrap();
    write.invok(mess);
}

///创建新的tcp服务器
pub fn new(address: &str, rm: Arc<RwLock<RoomMgr>>) {
    let sh = TcpServerHandler {
        sender: None,
        rm,
    };
    tcp_server::new(address, sh).unwrap();
}
///byte数组转换Packet
pub fn build_packet_mess_pt(mess: &MessPacketPt) -> Packet {
    //封装成packet
    let mut packet = Packet::new(mess.cmd);
    packet.set_data(&mess.write_to_bytes().unwrap()[..]);
    packet
}

///byte数组转换Packet
pub fn build_packet_bytes(bytes: &[u8]) -> Packet {
    let mut mpp = MessPacketPt::new();
    mpp.merge_from_bytes(bytes);

    //封装成packet
    let mut packet = Packet::new(mpp.cmd);
    packet.set_data(&mpp.write_to_bytes().unwrap()[..]);
    packet
}

///byte数组转换Packet
pub fn build_packet(mess: MessPacketPt) -> Packet {
    //封装成packet
    let mut packet = Packet::new(mess.cmd);
    packet.set_data(&mess.write_to_bytes().unwrap()[..]);
    packet
}


