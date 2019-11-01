pub mod bytebuf;
pub mod channel;
pub mod packet;
pub mod tcpsocket;
pub mod websocket;
use crate::mgr::game_mgr::GameMgr;
use protobuf::Message;
use std::io::Read;
use std::mem::transmute;
use std::net::{TcpListener, TcpStream};
use std::result::Result as ByteBufResult;
use std::sync::{Arc, Mutex};
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender as WsSender, Settings, WebSocket,
};

use crate::entity::contants::*;
use crate::entity::user::User;
use crate::entity::{Dao, Data};
use crate::net::bytebuf::ByteBuf;
use crate::net::channel::Channel;
use crate::net::packet::{Packet, PacketDes};
use crate::protos::base;
use crate::protos::base::{MessPacketPt, PlayerPt};
use crate::protos::message;
use crate::protos::message::MsgEnum_MsgCode::C_USER_LOGIN;
use crate::protos::message::MsgEnum_MsgCode::S_USER_LOGIN;
use crate::protos::protocol::{
    C_USER_LOGIN as C_USER_LOGIN_PROTO, S_USER_LOGIN as S_USER_LOGIN_PROTO,
};
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use protobuf::ProtobufEnum;
use serde_json::map::Entry::Vacant;
use std::convert::TryFrom;
use std::rc::Rc;
