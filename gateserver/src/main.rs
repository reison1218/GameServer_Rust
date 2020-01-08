mod mgr;
mod net;
mod protos;
mod prototools;
use crate::mgr::channelmgr::{new_tcp_client, ChannelMgr};
use crate::mgr::{
    channelmgr,
    thread_pool_mgr::ThreadPoolHandler,
    thread_pool_mgr::{MyThreadPool, ThreadPoolType},
};
use crate::protos::base;
use crate::prototools::proto;
use futures::executor::block_on;
use futures::future::join;
use futures::task::Poll;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use net::websocket::WebSocketHandler;
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use std::collections::HashMap;
use std::fs::File;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use threadpool::ThreadPool;
use ws::{Builder, Settings};

use std::sync::mpsc::{channel, Receiver, Sender};

use async_std::task;
use crossbeam::atomic::AtomicCell;
use std::borrow::BorrowMut;
use std::thread::sleep;

use async_std::task::spawn;

use futures::join;

mod entity;
#[macro_use]
extern crate lazy_static;

///初始化全局线程池
lazy_static! {
    //初始化线程池
    static ref THREAD_POOL: MyThreadPool = {
        let game_name = "game_name".to_string();
        let user_name = "user_name".to_string();
        let sys_name = "sys_name".to_string();
        let mtp = MyThreadPool::init(game_name, 4, user_name, 8, sys_name, 2);
        mtp
    };
    static ref i:  Arc<RwLock<test>> = {
        let mut arc: Arc<RwLock<test>> = Arc::new(RwLock::new(test { i: 0 }));
        arc
    };
}

fn test_async() {
    let a = async {
        loop {
            let d = Duration::from_secs(5);
            async_std::task::sleep(d).await;
            println!("123");
            let t = std::thread::current();
            println!("------{:?}", t.name().unwrap());
        }
    };

    let b = async {
        loop {
            let d = Duration::from_secs(2);
            async_std::task::sleep(d).await;

            println!("321");
            let t = std::thread::current();
            println!("=========={:?}", t.name().unwrap());
        }
    };

    let aa: task::JoinHandle<_> = task::spawn(a);
    let bb: task::JoinHandle<_> = task::spawn(b);
    block_on(aa);
    block_on(bb);
}
struct test {
    i: u32,
}
fn test_async_thread() {
    let a = async {
        for j in 0..10 {
            i.write().unwrap().i += 1;
            let d = Duration::from_secs(1);
            std::thread::sleep(d);
            println!(
                "{:?},{:?},{}",
                std::thread::current().id(),
                std::thread::current().name(),
                i.read().unwrap().i
            );
        }
    };
    //let i2 = i.clone();
    let b = async {
        for j in 0..10 {
            i.write().unwrap().i += 1;
            let d = Duration::from_secs(1);
            std::thread::sleep(d);
            println!(
                "{:?},{:?},{}",
                std::thread::current().id(),
                std::thread::current().name(),
                i.read().unwrap().i
            );
        }
    };

    let i3 = i.clone();
    let m = move || {
        for j in 0..10 {
            i3.write().unwrap().i += 1;
            let d = Duration::from_secs(1);
            std::thread::sleep(d);
            println!(
                "{:?},{:?},{}",
                std::thread::current().id(),
                std::thread::current().name(),
                i.read().unwrap().i
            );
        }
    };
    &THREAD_POOL.submit_game(m);
    let a = task::spawn(a);
    let b = task::spawn(b);
    block_on(a);
    block_on(b);

    let d = Duration::from_secs(5);
    std::thread::sleep(d);
    println!("{}", i.read().unwrap().i);
}

fn main() {
    let mut server_time = SystemTime::now();
    //初始化日志
    init_log();
    //初始化线程池
    let mut net_pool = ThreadPool::new_with_name("net_thread_pool".to_owned(), 4);

    //连接游戏服务器
    let cg = async {
        let mut cm = ChannelMgr::new();
        let cg = cm.connect_game();
        cg.await;
    };

    let cg = task::spawn(cg);

    //初始化websocket
    let is = init_web_socket();
    let is = task::spawn(is);

    info!(
        "服务器启动完成，监听端口：{},耗时：{}ms",
        9999,
        server_time.elapsed().unwrap().as_millis()
    );
    block_on(cg);
    block_on(is);
}

async fn init_web_socket() {
    let mut setting = Settings::default();
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
            game_net: new_tcp_client(),
        })
        .unwrap();
    let mut web_socket = server.listen("127.0.0.1:9999").unwrap();
    info!("websocket启动完成，监听：{}", 9999);
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
