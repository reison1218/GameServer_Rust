pub mod bytebuf;
pub mod channel;
pub mod packet;
pub mod tcpsocket;
pub mod websocket;
use crate::mgr::game_mgr::GameMgr;
use crate::protos::base::Test;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use protobuf::Message;
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::io::Read;
use std::mem::transmute;
use std::net::{TcpListener, TcpStream};
use std::result::Result as ByteBufResult;
use std::sync::{Arc, Mutex};
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender as WsSender, Settings, WebSocket,
};
