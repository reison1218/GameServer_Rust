mod mgr;
mod net;
mod protos;
mod prototools;
use crate::mgr::channelmgr::ChannelMgr;
use crate::protos::base;
use crate::prototools::proto;
use futures::executor::block_on;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use net::websocket::WebSocketHandler;
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::fs::File;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, Duration};
use threadpool::ThreadPool;
use ws::{Builder, Settings};
use crate::mgr::thread_pool_mgr::{MyThreadPool,ThreadPoolType};
use std::collections::HashMap;
use crate::mgr::thread_pool_mgr::ThreadPoolHandler;
use futures::task::Poll;
use futures::future::join;
use std::sync::mpsc::channel;

use crossbeam::atomic::AtomicCell;
use std::thread::sleep;
use std::borrow::BorrowMut;

mod entity;
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

struct Test {
    id: u32,
    name: String,
}

async fn test()->u32{
   1 as u32
}

fn main() {
//    let mut lock :Arc<AtomicCell<u32>>= Arc::new(AtomicCell::new(1 as u32));
//
//    let lock2 :AtomicCell<u32>= AtomicCell::new(1 as u32);
//
//    let mut lock_cp= lock.clone();
//    let m =move ||{
//        let d = Duration::from_secs(2000);
//        loop{
//           // sleep(d);
//            lock_cp.borrow_mut().store(2);
//        }
//
//    };
//    std::thread::spawn(m);
//    loop{
//        let d = Duration::from_secs(2000);
//        sleep(d);
//        println!("{}",lock.borrow_mut().load());
//    }




    let mut server_time = SystemTime::now();
    //初始化日志
    init_log();
    //初始化线程池
    let mut net_pool = ThreadPool::new_with_name("net_thread_pool".to_owned(), 4);

    //初始化网络，其中涉及到websocket和连接游戏服
    block_on(init_net());

    info!(
        "服务器启动完成，监听端口：{},耗时：{}ms",
        9999,
        server_time.elapsed().unwrap().as_millis()
    );
}

async fn init_net(){
    let cm  = init_tcp_client().await;
    init_web_socket(cm).await;
}

async fn init_tcp_client()->Arc<RwLock<ChannelMgr>>{
    let cm = ChannelMgr::new().await;
    let lock = RwLock::new(cm);
    let mut cm = Arc::new(lock);
    let cm = cm.clone();
    let cm_cp = cm.clone();
//    let cg =  move ||  {
//        cm.write().unwrap().connect_game();
//    };
//    THREAD_POOL.submit_game(cg);
    cm_cp
}

async fn init_web_socket(cm :Arc<RwLock<ChannelMgr>> ) {
    let mut setting = Settings::default();
    setting.max_connections = 2048;
    //websocket队列大小
    setting.queue_size = setting.max_connections * 2;
    //是否组合数据包
    setting.tcp_nodelay = true;
    let mut cm_cp = cm.clone();
    let mut server = Builder::new()
        .with_settings(setting)
        .build(|out| WebSocketHandler {
            ws: out,
            add: None,
            cm:cm.clone(),
        })
        .unwrap();

    let mut web_socket = server.listen("127.0.0.1:9999").unwrap();
    info!("websocket启动完成，监听：{}",9999);
    {
        cm_cp.write().unwrap().connect_game();
    }

}

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
