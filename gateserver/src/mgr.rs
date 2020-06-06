pub mod channel_mgr;
use crate::entity::gateuser::GateUser;
use log::{error, info, warn};
use std::collections::HashMap;
use std::net::TcpStream;
use tools::util::packet::Packet;
use ws::Sender as WsSender;
