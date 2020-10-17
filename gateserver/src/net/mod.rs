pub mod http;
pub mod tcp_client;
pub mod tcp_server;
pub mod websocket;
pub mod websocket_channel;
use crate::{CONF_MAP, REDIS_INDEX_USERS, REDIS_KEY_UID_2_PID};
use crate::{REDIS_KEY_USERS, REDIS_POOL};
use log::{debug, error, info, warn};
use protobuf::Message;
use std::net::TcpStream;
use std::sync::{Arc, MutexGuard};
use tools::tcp::ClientHandler;
use ws::{
    CloseCode, Error as WsError, Handler, Handshake, Message as WMessage, Result,
    Sender as WsSender,
};

use crate::mgr::channel_mgr::ChannelMgr;

use tools::util::packet::Packet;

use tools::cmd_code::GameCode;
use tools::protos::protocol::{C_USER_LOGIN, S_USER_LOGIN};

use serde_json::Value;
use std::str::FromStr;

///校验用户中心是否在线
fn check_uc_online(user_id: &u32) -> anyhow::Result<bool> {
    //校验用户中心是否登陆过，如果有，则不往下执行
    let mut redis_write = REDIS_POOL.write().unwrap();
    let pid: Option<String> = redis_write.hget(
        REDIS_INDEX_USERS,
        REDIS_KEY_UID_2_PID,
        user_id.to_string().as_str(),
    );
    if pid.is_none() {
        anyhow::bail!("this user_id is invalid!user_id:{}", user_id)
    }
    let pid = pid.unwrap();
    let res: Option<String> = redis_write.hget(0, "users", pid.as_str());
    if res.is_none() {
        anyhow::bail!("this user_id is invalid!user_id:{}", user_id)
    }
    let res = res.unwrap();
    let json = Value::from_str(res.as_str());
    match json {
        Ok(json_value) => {
            let bool_res = json_value["on_line"].as_bool();
            if bool_res.is_some() && bool_res.unwrap() {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
        Err(e) => anyhow::bail!("{:?}", e.to_string()),
    }
}

///校验内存是否在线，并做处理
fn check_mem_online(user_id: &u32, write: &mut MutexGuard<ChannelMgr>) -> bool {
    //校验内存是否已经登陆
    let gate_user = write.get_mut_user_channel_channel(user_id);
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
    let pid: Option<String> = redis_write.hget(
        REDIS_INDEX_USERS,
        REDIS_KEY_UID_2_PID,
        user_id.to_string().as_str(),
    );
    if pid.is_none() {
        return;
    }
    let pid = pid.unwrap();
    let res: Option<String> = redis_write.hget(REDIS_INDEX_USERS, REDIS_KEY_USERS, pid.as_str());
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

                let _: Option<u32> = redis_write.hset(
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
