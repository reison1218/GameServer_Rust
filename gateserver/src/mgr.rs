pub mod channel_mgr;
use crate::entity::gateuser::GateUser;
use log::{debug, error, info, warn, LevelFilter, Record};
use std::collections::HashMap;
use std::net::TcpStream;
use tools::util::packet::Packet;
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender as WsSender, Settings, WebSocket,
};
