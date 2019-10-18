mod mgr;
mod net;
mod protos;
mod prototools;
use crate::mgr::channelmgr::ChannelMgr;
use crate::mgr::gatemgr::GateMgr;
use crate::prototools::proto;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use net::websocket::WebSocketHandler;
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use threadpool::ThreadPool;
use ws::{Builder, Settings};

mod entity;

///初始化日志
fn init_log() {
    let mut log_time = SystemTime::now();
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

fn main() {
    //prototools::proto();
    let mut server_time = SystemTime::now();
    //初始化日志
    init_log();
    //初始化线程池
    let mut net_pool = ThreadPool::new_with_name("net_thread_pool".to_owned(), 4);

    //连接游戏服
    let mut cm = ChannelMgr::new();
    //初始化websocket
    init_web_socket();

    info!(
        "服务器启动完成，监听端口：{},耗时：{}ms",
        9999,
        server_time.elapsed().unwrap().as_millis()
    );
}

fn init_web_socket() {
    let mut setting = Settings::default();
    setting.max_connections = 2048;
    //websocket队列大小
    setting.queue_size = setting.max_connections * 2;
    //是否组合数据包
    setting.tcp_nodelay = true;
    let mut gate_mgr: Arc<std::sync::Mutex<GateMgr>> = Arc::new(Mutex::new(GateMgr::new()));

    let mut server = Builder::new()
        .with_settings(setting)
        .build(|out| WebSocketHandler {
            ws: out,
            add: None,
            gm: gate_mgr.clone(),
        })
        .unwrap();
    let mut web_socket = server.listen("127.0.0.1:9999").unwrap();
}
