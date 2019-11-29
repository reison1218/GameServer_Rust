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
use crate::mgr::thread_pool_mgr::{MyThreadPool, ThreadPoolHandler, ThreadPoolType};
use crate::net::{bytebuf::ByteBuf, tcpsocket, websocket, websocket::WebSocketHandler};
use crate::redispool::redistool;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use redis::{RedisResult, Value, Commands};
use scheduled_thread_pool::ScheduledThreadPool;
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::fs::File;

use std::convert::TryFrom;
use std::ops::Index;
use std::rc::Rc;
use std::sync::{atomic::AtomicUsize, Arc, Mutex, RwLock};
use std::thread::Thread;
use std::time::{Duration, SystemTime};
use threadpool::ThreadPool;
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender as WsSender, Sender, Settings, WebSocket,
};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::{json, value::Value::Object, Value as JsonValue};
use std::str::FromStr;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use std::time;
use futures::executor::block_on;

#[macro_use]
extern crate lazy_static;

///初始化全局线程池
lazy_static! {
    static ref THREAD_POOL: MyThreadPool = {
        let game_name = "game_name".to_string();
        let user_name = "user_name".to_string();
        let sys_name = "sys_name".to_string();
        let mtp = MyThreadPool::init(game_name, 4, user_name, 8, sys_name, 2);
        mtp
    };
    static ref DATA_MAP: RwLock<HashMap<String, Test>> = {
        let mut m: HashMap<String, Test> = HashMap::new();
        let mut lock = RwLock::new(m);
        lock
    };
}

const ID: &str = "id";
const TOKEN: &str = "token";


struct Test {
    id: u32,
    name: String,
}


async fn test1()->u32{
    println!("test1");
    println!("test1");
    let d = Duration::from_secs(5);
    std::thread::sleep(d);
    1
}

async fn test()->u32{
    let f = test1();

    println!("test");
    println!("test");
    f.await
}

///测试async/await
fn test_async_await(){
    let mut a = test();
    let a = block_on(a);

    let mut int = 1;
    let ay =async{
        int+=1;
    };
    block_on(ay);
    println!("int:{}",int);


    println!("future return u32:{}",a);
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

        //初始化websocket
        let mut setting = Settings::default();
        //websocket最大连接数
        setting.max_connections = 2048;
        //websocket队列大小
        setting.queue_size = setting.max_connections * 2;
        //是否组合数据包
        setting.tcp_nodelay = true;

        //初始化websocket
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
        //开始监听
        let mut web_socket =
            async{
                server.listen("127.0.0.1:9999").unwrap();
            };

        info!(
            "服务器启动完成，监听端口：{},耗时：{}ms",
            9999,
            server_time.elapsed().unwrap().as_millis()
        );
    //初始化tcpserver
    block_on(tcpsocket::new(game_mgr.clone()));
    //block_on(web_socket);
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

///保存玩家数据的定时器任务函数
fn save_timer(gm: Arc<RwLock<GameMgr>>) {
    let m = move || {
        let gm = gm.clone();
        loop {
            {
                let mut gm = gm.write().unwrap();
                gm.save_user();
            }

            let d = Duration::from_secs(60 * 5);
            std::thread::sleep(d);
        }
    };

    &THREAD_POOL.submit_game(m);
}
