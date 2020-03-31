pub mod channel_mgr;
pub mod cmd_code_mgr;
pub mod contants_mgr;
use crate::entity::gateuser::GateUser;
use crate::net::websocket_channel::WebSocketChannel;
use crate::THREAD_POOL;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;
use tcp::util::packet::{Packet, PacketDes};
use threadpool::ThreadPool;
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender as WsSender, Settings, WebSocket,
};
