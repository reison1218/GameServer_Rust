use super::*;
use std::collections::HashMap;
use tools::tcp::TcpSender;
use tools::protos::base::MessPacketPt;
use std::collections::hash_map::RandomState;
use crate::entity::NetClient::NetClient;

#[derive(Debug)]
pub struct RegisterMgr {
    //gateid->roomid绑定map
    pub g2r: HashMap<u32, u32>,
    //userid->gateid绑定map
    pub u2g: HashMap<u32, u32>,
    //gateserver客户端
    pub gate_channel: HashMap<u32, NetClient>,
    //room客户端
    pub room_channel:HashMap<u32, NetClient>,
    //命令map
    pub cmd_map: HashMap<u32, fn(&mut RoomMgr, MessPacketPt), RandomState>, //命令管理
}