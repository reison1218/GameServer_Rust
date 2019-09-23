pub mod bytebuf;
pub mod channel;
pub mod packet;
pub mod websocket;
use crate::protos::base::Test;
use crate::mgr::gatemgr::GateMgr;
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
