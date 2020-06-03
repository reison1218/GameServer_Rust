pub mod http;
pub mod tcp_server;
use crate::entity::user_info::User;
use crate::entity::Dao;
use crate::mgr::game_mgr::GameMgr;
use log::{error, info, warn};
use std::io::Read;
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
use tools::tcp::TcpSender;
use tools::util::bytebuf::ByteBuf;
use ws::Sender;
