mod db;
mod entity;
mod helper;
mod mgr;
mod net;
mod template;
use crate::db::dbtool::DbPool;
use crate::entity::{Dao, Entity, EntityData};
use crate::mgr::game_mgr::GameMgr;
use crate::net::http::{SavePlayerHttpHandler, StopPlayerHttpHandler};
use crate::net::tcp_server;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fs::File;
use tools::thread_pool::{MyThreadPool, ThreadPoolHandler, ThreadPoolType};

use std::convert::TryFrom;
use std::ops::Index;
use std::rc::Rc;
use std::sync::{atomic::AtomicUsize, Arc, Mutex, RwLock};
use std::thread::Thread;
use std::time::{Duration, SystemTime};
use threadpool::ThreadPool;

use async_std::task;
use chrono::{DateTime, Datelike, Local, NaiveDateTime, Timelike, Utc};
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use serde_json::{json, value::Value::Object, Value as JsonValue};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::time;

use crate::entity::user_info::User;
use crate::mgr::timer_mgr;
use crate::template::templates::Templates;
use futures::AsyncWriteExt;
use mysql::prelude::ToValue;
use std::cell::RefCell;
use std::env;
use std::sync::mpsc::{Receiver, Sender};
use tools::conf::Conf;
use tools::http::HttpServerHandler;
use tools::my_log::init_log;
use tools::redis_pool::RedisPoolTool;
use tools::util::bytebuf::ByteBuf;

#[macro_use]
extern crate lazy_static;

///初始化全局线程池
lazy_static! {

    ///线程池
    static ref THREAD_POOL: MyThreadPool = {
        let game_name = "game_name".to_string();
        let user_name = "user_name".to_string();
        let sys_name = "sys_name".to_string();
        let mtp = MyThreadPool::init(game_name, 8, user_name, 8, sys_name, 2);
        mtp
    };

    ///数据库链接池
    static ref DB_POOL: DbPool = {
        let db_pool = DbPool::new();
        db_pool
    };

    ///配置文件
    static ref CONF_MAP: Conf = {
        let path = env::current_dir().unwrap();
        let mut str = path.as_os_str().to_str().unwrap();
        let res = str.to_string()+"/config/config.conf";
        let conf = Conf::init(res.as_str());
        conf
    };

    ///静态配置文件
    static ref TEMPLATES: Templates = {
        let path = env::current_dir().unwrap();
        let mut str = path.as_os_str().to_str().unwrap();
        let res = str.to_string()+"/template";
        let conf = Templates::new(res.as_str());
        conf
    };

    ///reids客户端
    static ref REDIS_POOL:Arc<RwLock<RedisPoolTool>>={
        let add: &str = CONF_MAP.get_str("redis_add");
        let pass: &str = CONF_MAP.get_str("redis_pass");
        let redis = RedisPoolTool::init(add,pass);
        let redis:Arc<RwLock<RedisPoolTool>> = Arc::new(RwLock::new(redis));
        redis
    };
}

///程序主入口,主要作用是初始化日志，数据库连接，redis连接，线程池，websocket，http
fn main() {
    let mut game_mgr: Arc<RwLock<GameMgr>> = Arc::new(RwLock::new(GameMgr::new()));

    let info_log = CONF_MAP.get_str("info_log_path");
    let error_log = CONF_MAP.get_str("error_log_path");

    //初始化日志模块
    init_log(info_log, error_log);

    //初始化定时器任务管理
    timer_mgr::init(game_mgr.clone());

    //初始化http服务端
    init_http_server(game_mgr.clone());

    //初始化tcp服务端
    init_tcp_server(game_mgr.clone());
}

///初始化http服务端
fn init_http_server(gm: Arc<RwLock<GameMgr>>) {
    let mut http_vec: Vec<Box<dyn HttpServerHandler>> = Vec::new();
    http_vec.push(Box::new(SavePlayerHttpHandler::new(gm.clone())));
    http_vec.push(Box::new(StopPlayerHttpHandler::new(gm.clone())));
    let httpPort: &str = CONF_MAP.get_str("http_port");
    async_std::task::spawn(tools::http::http_server(httpPort, http_vec));
}

///init tcp server
fn init_tcp_server(gm: Arc<RwLock<GameMgr>>) {
    let tcpPort: &str = CONF_MAP.get_str("tcp_port");
    tcp_server::new(tcpPort, gm);
}
