mod entity;
mod mgr;
mod net;
use crate::mgr::channel_mgr::ChannelMgr;
use crate::net::tcp_client::TcpClientHandler;
use futures::executor::block_on;
use futures::future::join;
use futures::task::Poll;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use net::websocket::WebSocketHandler;
use std::collections::HashMap;
use std::fs::File;
use std::net::TcpStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};
use threadpool::ThreadPool;
use tools::conf::Conf;
use tools::protos::base;
use tools::thread_pool::ThreadPoolHandler;
use ws::{
    connect, Builder, CloseCode, Error, Factory, Handler, Handshake, Message as WMessage, Request,
    Response, Result, Sender as WsSender, Settings, WebSocket,
};

use std::sync::mpsc::{channel, Receiver, Sender};

use async_std::task;
use std::borrow::BorrowMut;
use std::thread::sleep;

use async_std::task::spawn;

use crate::net::tcp_client::TcpClientType;
use crate::net::tcp_server;
use crate::net::websocket::ClientSender;
use futures::join;
use protobuf::Message;
use std::env;
use std::io::Read;
use tools::my_log::init_log;
use tools::redis_pool::RedisPoolTool;
use tools::tcp::ClientHandler;
use tools::thread_pool::MyThreadPool;
use tools::util::bytebuf::ByteBuf;
use tools::util::packet::{Packet, PacketDes};

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
        let path = env::current_dir().unwrap();
        let mut str = path.as_os_str().to_str().unwrap();
        let res = str.to_string()+"/config/config.conf";
        let conf = Conf::init(res.as_str());
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

    static ref ID:Arc<RwLock<Test>> ={
        let t = Test{id:1011000000};
        let mut arc: Arc<RwLock<Test>> = Arc::new(RwLock::new(t));
        arc
    };
}

struct Test {
    pub id: u32,
}

fn main() {
    //获得日志配置
    let info_log = CONF_MAP.get_str("info_log_path");
    let error_log = CONF_MAP.get_str("error_log_path");

    //初始化日志
    init_log(info_log, error_log);

    //创建核心结构体，channel管理器，因为涉及到多线程异步，所以创建结构体的arc引用计数器指针
    let mut cm = Arc::new(RwLock::new(ChannelMgr::new()));

    //连接游戏服务器
    init_game_tcp_connect(cm.clone());

    //连接房间服务器
    init_room_tcp_connect(cm.clone());

    //初始化与客户端通信的模块
    init_net_server(cm);
}

///初始化网络服务这块
fn init_net_server(cm: Arc<RwLock<ChannelMgr>>) {
    //获取通信模块
    let net_module = CONF_MAP.get_str("net_module");
    match net_module {
        "tcp" => {
            //初始化tcp服务端
            init_tcp_server(cm);
        }
        "webSocket" => {
            //初始化websocket
            init_web_socket(cm);
        }
        _ => {
            //初始化tcp服务端
            init_tcp_server(cm);
        }
    }
}

///初始化游戏服务器tcp客户端链接
fn init_game_tcp_connect(cp: Arc<RwLock<ChannelMgr>>) {
    let game = async {
        let mut tch = TcpClientHandler::new(cp, TcpClientType::GameServer);
        let address = CONF_MAP.get_str("game_port");
        info!("开始链接游戏服:{:?}", address);
        tch.on_read(address.to_string());
    };
    async_std::task::spawn(game);
}

///初始化房间服务器tcp客户端链接
fn init_room_tcp_connect(cp: Arc<RwLock<ChannelMgr>>) {
    let room = async {
        let mut tch = TcpClientHandler::new(cp, TcpClientType::RoomServer);
        let address = CONF_MAP.get_str("room_port");
        info!("开始链接房间服:{:?}", address);
        tch.on_read(address.to_string());
    };
    async_std::task::spawn(room);
}

///初始化tcp服务端
fn init_tcp_server(cm: Arc<RwLock<ChannelMgr>>) {
    let str = CONF_MAP.get_str("tcp_port");
    tcp_server::new(str, cm);
}

///初始化websocket
fn init_web_socket(cp: Arc<RwLock<ChannelMgr>>) {
    let mut setting = Settings::default();
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
    let str = CONF_MAP.get_str("web_socket_port");
    let mut web_socket = server.listen(str).unwrap();
}
