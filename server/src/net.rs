pub mod http;
pub mod tcp_server;
use crate::entity::contants::*;
use crate::entity::user_info::User;
use crate::entity::{Dao, EntityData};
use crate::mgr::game_mgr::GameMgr;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use serde_json::map::Entry::Vacant;
use std::convert::TryFrom;
use std::io::Read;
use std::mem::transmute;
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::result::Result as ByteBufResult;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
use tools::protos::protocol::{
    C_USER_LOGIN as C_USER_LOGIN_PROTO, S_USER_LOGIN as S_USER_LOGIN_PROTO,
};
use tools::thread_pool::ThreadPoolHandler;
use tools::util::packet::{Packet, PacketDes};
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender as WsSender, Settings, WebSocket,
};

use crate::db::table_contants::*;
use crate::THREAD_POOL;
use std::sync::{MutexGuard, RwLock, RwLockWriteGuard};
use tools::protos::base::PlayerPt;
use tools::protos::server_protocol::MessPacketPt;
use tools::util::bytebuf::ByteBuf;
use ws::Sender;
