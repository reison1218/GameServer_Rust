pub mod channel;
pub mod tcpsocket;
use crate::entity::contants::*;
use crate::entity::user::User;
use crate::entity::{Dao, Data};
use crate::mgr::game_mgr::GameMgr;
use crate::net::channel::Channel;
use crate::protos::base;
use crate::protos::message;
use crate::protos::message::MsgEnum_MsgCode::C_USER_LOGIN;
use crate::protos::message::MsgEnum_MsgCode::S_USER_LOGIN;
use crate::protos::protocol::{
    C_USER_LOGIN as C_USER_LOGIN_PROTO, S_USER_LOGIN as S_USER_LOGIN_PROTO,
};
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use protobuf::Message;
use protobuf::ProtobufEnum;
use serde_json::map::Entry::Vacant;
use std::convert::TryFrom;
use std::io::Read;
use std::mem::transmute;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::result::Result as ByteBufResult;
use std::sync::{Arc, Mutex};
use tcp::thread_pool::ThreadPoolHandler;
use tcp::util::packet::{Packet, PacketDes};
use threadpool::ThreadPool;
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender as WsSender, Settings, WebSocket,
};

use crate::protos::base::{MessPacketPt, PlayerPt};
use crate::THREAD_POOL;
use std::sync::{MutexGuard, RwLock, RwLockWriteGuard};
use tcp::util::bytebuf::ByteBuf;
use ws::Sender;
