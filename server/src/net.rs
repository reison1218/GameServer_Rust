pub mod http;
pub mod tcp_server;
use crate::entity::user_info::User;
use crate::mgr::game_mgr::GameMgr;
use log::{error, info, warn};
use std::sync::Arc;
use tools::protos::protocol::{
    C_USER_LOGIN as C_USER_LOGIN_PROTO, S_USER_LOGIN as S_USER_LOGIN_PROTO,
};
use tools::util::packet::Packet;

use std::sync::RwLock;
use tools::protos::base::PlayerPt;
