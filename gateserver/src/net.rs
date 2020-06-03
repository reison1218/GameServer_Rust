pub mod tcp_client;
pub mod tcp_server;
pub mod websocket;
pub mod websocket_channel;
use crate::CONF_MAP;
use crate::REDIS_POOL;
use log::{debug, error, info, warn};
use protobuf::Message;
use std::net::TcpStream;
use std::sync::{Arc, RwLockWriteGuard};
use tools::tcp::ClientHandler;
use ws::{
    Builder, CloseCode, Error as WsError, Factory, Handler, Handshake, Message as WMessage,
    Request, Response, Result, Sender as WsSender, Settings, WebSocket,
};

use crate::mgr::channel_mgr::ChannelMgr;
use std::sync::RwLock;

use tools::util::packet::Packet;

use tools::cmd_code::GameCode;
use tools::protos::protocol::{C_USER_LOGIN, S_USER_LOGIN};

use serde_json::Value;
use std::str::FromStr;

///校验用户中心是否在线
fn check_uc_online(user_id: &u32) -> bool {
    //校验用户中心是否登陆过，如果有，则不往下执行
    let mut redis_write = REDIS_POOL.write().unwrap();
    let pid: Option<String> = redis_write.hget(1, "uid_2_pid", user_id.to_string().as_str());
    if pid.is_none() {
        return false;
    }
    let pid = pid.unwrap();
    let res: Option<String> = redis_write.hget(0, "users", pid.as_str());
    if res.is_none() {
        return false;
    }
    let res = res.unwrap();
    let json = Value::from_str(res.as_str());
    match json {
        Ok(json_value) => {
            let bool_res = json_value["on_line"].as_bool();
            if bool_res.is_some() && bool_res.unwrap() {
                return true;
            }
        }
        Err(e) => {
            error!("{:?}", e);
            return false;
        }
    }
    false
}

///校验内存是否在线，并做处理
fn check_mem_online(user_id: &u32, write: &mut RwLockWriteGuard<ChannelMgr>) -> bool {
    //校验内存是否已经登陆
    let mut gate_user = write.get_mut_user_channel_channel(user_id);
    let mut res: bool = false;
    //如果有，则执行T下线
    if gate_user.is_some() {
        // let token = gate_user.as_mut().unwrap().get_tcp_ref().token;
        // write.close_remove(&token);
        res = true;
    }
    res
}

fn modify_redis_user(user_id: u32, is_login: bool) {
    let mut redis_write = REDIS_POOL.write().unwrap();
    let pid: Option<String> = redis_write.hget(1, "uid_2_pid", user_id.to_string().as_str());
    if pid.is_none() {
        return;
    }
    let pid = pid.unwrap();
    let res: Option<String> = redis_write.hget(0, "users", pid.as_str());
    if res.is_none() {
        return;
    }
    let res = res.unwrap();
    let json = Value::from_str(res.as_str());

    match json {
        Ok(mut json_value) => {
            let json_res = json_value.as_object_mut();
            if json_res.is_some() {
                json_res
                    .unwrap()
                    .insert("on_line".to_owned(), Value::from(is_login));

                let res: Option<u32> = redis_write.hset(
                    0,
                    "users",
                    pid.to_string().as_str(),
                    json_value.to_string().as_str(),
                );
            }
        }
        Err(e) => {
            error!("{:?}", e);
        }
    }
}
