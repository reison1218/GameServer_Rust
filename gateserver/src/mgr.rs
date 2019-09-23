pub mod channelmgr;
pub mod gatemgr;
use crate::entity::gateuser::GateUser;
use crate::net::packet::Packet;
use crate::net::packet::PacketDes;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender as WsSender, Settings, WebSocket,
};
