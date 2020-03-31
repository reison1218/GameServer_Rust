mod mgr;
mod net;
mod protos;
mod prototools;
use crate::mgr::channel_mgr::{build_Mess, ChannelMgr};
use crate::net::tcp_client::TcpClientHandler;
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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use tcp::conf::Conf;
use tcp::thread_pool::ThreadPoolHandler;
use threadpool::ThreadPool;
use ws::{
    connect, Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request,
    Response, Result, Sender as WsSender, Settings, WebSocket,
};

use std::sync::mpsc::{channel, Receiver, Sender};

use async_std::task;
use std::borrow::BorrowMut;
use std::thread::sleep;

use async_std::task::spawn;

use crate::net::tcp_server;
use crate::net::websocket::ClientSender;
use crate::protos::base::MessPacketPt;
use crate::protos::message::MsgEnum_MsgCode::S_USER_LOGIN;
use futures::join;
use protobuf::Message;
use std::io::Read;
use tcp::tcp::ClientHandler;
use tcp::thread_pool::MyThreadPool;
use tcp::util::bytebuf::ByteBuf;
use tcp::util::packet::{Packet, PacketDes};

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
    static ref CONF_MAP: Conf = {
        let conf = Conf::init("/Users/tangjian/git/MyRust/gateserver/configs/config.conf");
        conf
    };

    static ref ID:Arc<RwLock<Test>> ={
        let t = Test{id:0};
        let mut arc: Arc<RwLock<Test>> = Arc::new(RwLock::new(t));
        arc
    };
}

struct Test {
    pub id: u32,
}

fn main() {
    let mut server_time = SystemTime::now();
    //初始化日志
    init_log();
    //初始化线程池
    let mut net_pool = ThreadPool::new_with_name("net_thread_pool".to_owned(), 4);

    let mut cm = Arc::new(RwLock::new(ChannelMgr::new()));
    //连接游戏服务器
    let cg = init_tcp_connect(cm.clone());
    let cg = task::spawn(cg);

    //初始化websocket
    //    let is = init_web_socket(cm.clone());
    //    let is = task::spawn(is);

    let is = init_tcpserver(cm.clone());
    let is = task::spawn(is);
    info!(
        "gate启动完成!耗时：{}ms",
        server_time.elapsed().unwrap().as_millis()
    );
    block_on(cg);
    block_on(is);
}

///初始化tcp
async fn init_tcp_connect(cp: Arc<RwLock<ChannelMgr>>) {
    let mut tch = TcpClientHandler::new(cp);
    let address = CONF_MAP.get_str("gamePort");
    tch.on_read(address.to_string());
}

///初始化tcp服务端
async fn init_tcpserver(cm: Arc<RwLock<ChannelMgr>>) {
    let str = CONF_MAP.get_str("websocketPort");
    tcp_server::new(str, cm);
}

///初始化websocket
async fn init_web_socket(cp: Arc<RwLock<ChannelMgr>>) {
    let mut setting = Settings::default();
    setting.tcp_nodelay = true;
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
                cm: cp.clone(),
            }
        })
        .unwrap();
    let str = CONF_MAP.get_str("websocketPort");
    let mut web_socket = server.listen(str).unwrap();
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
            File::create("/tmp/gateLog/info.log").unwrap(),
        ),
        WriteLogger::new(
            LevelFilter::Error,
            config.build(),
            File::create("/tmp/gateLog/error.log").unwrap(),
        ),
    ])
    .unwrap();
    info!(
        "日志模块初始化完成！耗时{}ms",
        log_time.elapsed().unwrap().as_millis()
    );
}

///byte数组转换Packet
pub fn build_packet(mess: MessPacketPt) -> Packet {
    //封装成packet
    let mut packet = Packet::new(mess.cmd);
    packet.set_data(&mess.write_to_bytes().unwrap()[..]);
    packet
}
