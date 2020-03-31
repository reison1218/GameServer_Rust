mod db;
mod entity;
mod mgr;
mod net;
mod protos;
mod prototools;
mod redispool;
use crate::db::dbtool::DbPool;
use crate::entity::Dao;
use crate::mgr::game_mgr::GameMgr;
use crate::net::tcpsocket;
use crate::redispool::redistool;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use redis::{Commands, RedisResult, Value};
use scheduled_thread_pool::ScheduledThreadPool;
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fs::File;
use tcp::thread_pool::{MyThreadPool, ThreadPoolHandler, ThreadPoolType};

use std::convert::TryFrom;
use std::ops::Index;
use std::rc::Rc;
use std::sync::{atomic::AtomicUsize, Arc, Mutex, RwLock};
use std::thread::Thread;
use std::time::{Duration, SystemTime};
use threadpool::ThreadPool;
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender as WsSender, Settings, WebSocket,
};

use async_std::task;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use serde_json::{json, value::Value::Object, Value as JsonValue};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::time;

use crate::entity::user::User;
use crate::protos::message::MsgEnum_MsgCode::C_USER_LOGIN;
use futures::AsyncWriteExt;
use mysql::prelude::ToValue;
use protobuf::Message;
use std::cell::RefCell;
use std::sync::mpsc::{Receiver, Sender};
use tcp::conf::Conf;
use tcp::util::bytebuf::ByteBuf;
use tcp::util::packet::{Packet, PacketDes};

#[macro_use]
extern crate lazy_static;

///初始化全局线程池
lazy_static! {
    static ref THREAD_POOL: MyThreadPool = {
        let game_name = "game_name".to_string();
        let user_name = "user_name".to_string();
        let sys_name = "sys_name".to_string();
        let mtp = MyThreadPool::init(game_name, 8, user_name, 8, sys_name, 2);
        mtp
    };
    static ref CONF_MAP: Conf = {
        let conf = Conf::init("/Users/tangjian/git/MyRust/server/configs/config.conf");
        conf
    };
    static ref DATA_MAP: RwLock<HashMap<String, Test>> = {
        let mut m: HashMap<String, Test> = HashMap::new();
        let mut lock = RwLock::new(m);
        lock
    };
    static ref ID: Arc<RwLock<u32>> = {
        let mut arc: Arc<RwLock<u32>> = Arc::new(RwLock::new(0 as u32));
        arc
    };
}

struct Test {
    id: u32,
}

pub fn test_mysql2() {
    let mut pool = DbPool::new();
    let mut str = "insert into t_u_player(user_id,content) values(:user_id,:content)";
    let mut v: Vec<mysql::Value> = Vec::new();
    v.push(mysql::Value::Int(7));
    let mut local = chrono::Utc::now();
    let mut map = serde_json::map::Map::new();
    let ss = local.format("%Y-%m-%dT%H:%M:%S").to_string();
    let jv = JsonValue::String(ss);
    map.insert("lastLoginTime".to_string(), jv);
    let mut jv = JsonValue::Object(map);
    v.push(jv.to_value());
    let re = pool.exe_sql(str, Some(v));
    if re.is_err() {
        println!("{:?}", re.err().unwrap().to_string());
    }
    let user = User::query(7, &mut pool);
    pool.exe_sql("delete from t_u_player where user_id = 7", None);
}

fn fn_life_time<'b, 'a: 'b>(str1: &'a str, str2: &'b str) -> &'a str {
    str1
}

///程序主入口,主要作用是初始化日志，数据库连接，redis连接，线程池，websocket
fn main() {
    let mut server_time = time::SystemTime::now();
    //初始化日志模块
    init_log();
    info!("开始测试mysql");
    //初始化数据库连接池
    let mut db_pool = DbPool::new();
    //gameMgr引用计数器
    let mut game_mgr: Arc<RwLock<GameMgr>> = Arc::new(RwLock::new(GameMgr::new(db_pool)));

    info!("开始测试redis");
    redistool::test_api();

    //初始化定时器线程池
    save_timer(game_mgr.clone());

    DATA_MAP
        .write()
        .unwrap()
        .borrow_mut()
        .insert("1".to_string(), Test { id: 1 });

    info!(
        "服务器启动完成!耗时：{}ms",
        server_time.elapsed().unwrap().as_millis()
    );
    let tcpPort: &str = CONF_MAP.get_str("tcpPort");

    //初始化tcpserver
    init_tcpserver(tcpPort, game_mgr.clone());
}

///初始化tcp服务端
fn init_tcpserver(address: &str, gm: Arc<RwLock<GameMgr>>) {
    tcpsocket::new(address, gm);
}

///初始化日志
fn init_log() {
    let mut log_time = time::SystemTime::now();
    let mut config = simplelog::ConfigBuilder::new();
    config.set_time_format_str("%Y-%m-%d %H:%M:%S");
    config.set_target_level(LevelFilter::Error);
    config.set_location_level(LevelFilter::Error);
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, config.build(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            LevelFilter::Info,
            config.build(),
            File::create("/tmp/serverLog/info.log").unwrap(),
        ),
        WriteLogger::new(
            LevelFilter::Error,
            config.build(),
            File::create("/tmp/serverLog/error.log").unwrap(),
        ),
    ])
    .unwrap();
    info!(
        "日志模块初始化完成！耗时{}ms",
        log_time.elapsed().unwrap().as_millis()
    );
}

///保存玩家数据的定时器任务函数
fn save_timer(gm: Arc<RwLock<GameMgr>>) {
    let m = move || {
        let gm = gm.clone();
        loop {
            let mut gm = gm.write().unwrap();
            gm.save_user();
            std::mem::drop(gm);
            let d = Duration::from_secs(60 * 5);
            std::thread::sleep(d);
        }
    };

    &THREAD_POOL.submit_game(m);
}
