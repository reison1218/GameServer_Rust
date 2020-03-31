pub mod tcp_client;
pub mod tcp_server;
pub mod websocket;
pub mod websocket_channel;
use crate::CONF_MAP;
use std::net::TcpStream;
use tcp::tcp::ClientHandler;
use tcp::util::bytebuf::ByteBuf;
//use crate::protos::base::Test;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use protobuf::Message;
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::mem::transmute;
use std::result::Result as ByteBufResult;
use std::sync::{Arc, Mutex};
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender as WsSender, Settings, WebSocket,
};

use crate::mgr::channel_mgr::ChannelMgr;
use crate::mgr::cmd_code_mgr::{GAME_MAX, GAME_MIN, ROOM_MAX, ROOM_MIN};
use crate::protos::base::MessPacketPt;
use crate::THREAD_POOL;
use std::borrow::Borrow;
use std::sync::RwLock;

use tcp::util::packet::{Packet, PacketDes};

use crate::protos::message::MsgEnum_MsgCode::C_USER_LOGIN;
use crate::protos::message::MsgEnum_MsgCode::S_USER_LOGIN;
use crate::protos::protocol::{
    C_USER_LOGIN as C_USER_LOGIN_PROTO, S_USER_LOGIN as S_USER_LOGIN_PROTO,
};
