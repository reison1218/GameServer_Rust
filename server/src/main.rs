mod db;
mod entity;
mod mgr;
mod net;
mod protos;
mod prototools;
mod redispool;
use crate::db::dbtool::DbPool;
use crate::entity::Entity;
use crate::mgr::game_mgr::GameMgr;
use crate::net::bytebuf::ByteBuf;
use crate::net::tcpsocket;
use crate::net::websocket;
use crate::net::websocket::WebSocketHandler;
use crate::protos::base::Test;
use crate::redispool::redistool;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use protobuf::Message;
use redis::RedisResult;
use redis::Value;
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::sync::Mutex;
use std::time;
use std::time::SystemTime;
use threadpool::ThreadPool;
use ws::{
    Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request, Response,
    Result, Sender, Settings, WebSocket,
};

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
            File::create("/tmp/info.log").unwrap(),
        ),
        WriteLogger::new(
            LevelFilter::Error,
            config.build(),
            File::create("/tmp/error.log").unwrap(),
        ),
    ])
    .unwrap();
    info!(
        "日志模块初始化完成！耗时{}ms",
        log_time.elapsed().unwrap().as_millis()
    );
}

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
    tcpsocket::new();

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
        .build(|out| WebSocketHandler {
            ws: out,
            add: None,
            gm: game_mgr.clone(),
        })
        .unwrap();
    let mut web_socket = server.listen("127.0.0.1:9999").unwrap();
    info!(
        "服务器启动完成，监听端口：{},耗时：{}ms",
        9999,
        server_time.elapsed().unwrap().as_millis()
    );
}
