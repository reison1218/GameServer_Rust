pub mod bytebuf;
pub mod channel;
pub mod packet;
pub mod websocket;
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

use crate::mgr::channelmgr::ChannelMgr;
use crate::mgr::cmd_code_mgr::{GAME_MAX, GAME_MIN, ROOM_MAX, ROOM_MIN};
use crate::mgr::thread_pool_mgr::ThreadPoolHandler;
use crate::net::packet::{Packet, PacketDes};
use crate::protos::base::MessPacketPt;
use crate::THREAD_POOL;
use std::borrow::Borrow;
use std::sync::RwLock;
