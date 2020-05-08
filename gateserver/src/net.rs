pub mod tcp_client;
pub mod tcp_server;
pub mod websocket;
pub mod websocket_channel;
use crate::CONF_MAP;
use std::net::TcpStream;
use tools::tcp::ClientHandler;
use tools::util::bytebuf::ByteBuf;
//use crate::protos::base::Test;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use protobuf::Message;
use std::mem::transmute;
use std::result::Result as ByteBufResult;
use std::sync::{Arc, Mutex};
use ws::{
    Builder, CloseCode, Error as WsError, Factory, Handler, Handshake, Message as WMessage,
    Request, Response, Result, Sender as WsSender, Settings, WebSocket,
};

use crate::mgr::channel_mgr::ChannelMgr;
use crate::THREAD_POOL;
use std::borrow::Borrow;
use std::sync::RwLock;
use tools::protos::server_protocol::MessPacketPt;

use tools::util::packet::{Packet, PacketDes};

use tools::cmd_code::GameCode;
use tools::protos::protocol::{C_USER_LOGIN, S_USER_LOGIN};

pub fn bytes_to_mess_packet_pt(mess: &[u8]) -> MessPacketPt {
    let mut bb = ByteBuf::from(mess);
    let mut packet = Packet::from(bb);
    let mut mp = MessPacketPt::new();
    mp.set_user_id(packet.get_user_id().unwrap());
    mp.set_cmd(packet.get_cmd());
    mp.set_is_client(true);
    mp.set_is_broad(false);
    mp.set_data(packet.get_data_vec());
    mp
}
