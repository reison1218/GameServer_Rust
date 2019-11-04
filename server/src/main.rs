mod db;
mod entity;
mod mgr;
mod net;
mod protos;
mod prototools;
mod redispool;
use crate::db::dbtool::DbPool;
use crate::entity::Dao;
use crate::mgr::channel_mgr::ChannelMgr;
use crate::mgr::game_mgr::GameMgr;
use crate::net::bytebuf::ByteBuf;
use crate::net::tcpsocket;
use crate::net::websocket;
use crate::net::websocket::WebSocketHandler;
use crate::redispool::redistool;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use redis::RedisResult;
use redis::Value;
use scheduled_thread_pool::ScheduledThreadPool;
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fs::File;

use std::convert::TryFrom;
use std::ops::Index;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::Thread;
use std::time;
use std::time::{Duration, SystemTime};
use threadpool::ThreadPool;
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender as WsSender, Sender, Settings, WebSocket,
};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::value::Value::Object;
use serde_json::{json, Value as JsonValue};
use std::str::FromStr;
use std::sync::mpsc::channel;

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
            File::create("F:/rustLog/info.log").unwrap(),
        ),
        WriteLogger::new(
            LevelFilter::Error,
            config.build(),
            File::create("F:/rustLog/error.log").unwrap(),
        ),
    ])
    .unwrap();
    info!(
        "日志模块初始化完成！耗时{}ms",
        log_time.elapsed().unwrap().as_millis()
    );
}

//测试代码开始
struct test {
    pub data: JsonValue,
}

const id: &str = "id";
const token: &str = "token";

static mut id2: &str = "id2";

fn test_json() {
    let mut db_pool = DbPool::new();
    let sql = "select * from t_u_player";
    let mut re = db_pool.exe_sql(sql, None);
    let mut re = re.unwrap();
    let mut data = (0, "".to_string());
    for _qr in re {
        data = mysql::from_row(_qr.unwrap());
        let mut js: JsonValue = serde_json::from_str(data.1.as_str()).unwrap();

        let mut map = js.as_object_mut();
        if map.is_none() {
            println!("map is none!");
            continue;
        }

        let mut map = map.unwrap();
        let str = map.get("ctime").unwrap();
        let str = str.as_str().unwrap();
        println!("{:?}", str);
        let time = "2015-09-18T23:56:04".parse::<NaiveDateTime>();
        let t = time.unwrap();
        println!("{:?}", t.to_string());
    }
}
//测试代码结束

///程序主入口,主要作用是初始化日志，数据库连接，redis连接，线程池，websocket
fn main() {
    let mut server_time = time::SystemTime::now();
    ///初始化日志模块
    init_log();

    info!("开始测试mysql");
    let mut db_pool = DbPool::new();
    let mut game_mgr: Arc<std::sync::Mutex<GameMgr>> = Arc::new(Mutex::new(GameMgr::new(db_pool)));

    info!("开始测试redis");
    redistool::test_api();

    let mut size = 0;

    //初始化线程池
    let mut net_pool = ThreadPool::new_with_name("net_thread_pool".to_owned(), 4);

    //初始化tcpserver
    //tcpsocket::new();

    //初始化定时器线程池
    let gm = game_mgr.clone();
    save_timer(gm, &mut net_pool);

    //初始化websocket
    let mut setting = Settings::default();
    //websocket最大连接数
    setting.max_connections = 2048;
    //websocket队列大小
    setting.queue_size = setting.max_connections * 2;
    //是否组合数据包
    setting.tcp_nodelay = true;

    let mut server = Builder::new()
        .with_settings(setting)
        .build(|out| {
            let mut arc: Arc<WsSender> = Arc::new(out);
            WebSocketHandler {
                ws: arc,
                add: None,
                gm: game_mgr.clone(),
            }
        })
        .unwrap();
    let mut web_socket = server.listen("127.0.0.1:9999").unwrap();
    info!(
        "服务器启动完成，监听端口：{},耗时：{}ms",
        9999,
        server_time.elapsed().unwrap().as_millis()
    );
}

fn save_timer(gm: Arc<Mutex<GameMgr>>, net_pool: &mut ThreadPool) {
    let m = move || loop {
        {
            let gm = gm.clone();
            let mut gm = gm.lock().unwrap();
            gm.save_user();
        }
        let d = Duration::from_secs(60 * 5);
        std::thread::sleep(d);
    };
    net_pool.execute(m);
}
