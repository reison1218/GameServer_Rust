mod auth;
mod entity;
mod mgr;
mod net;
use crate::mgr::channel_mgr::ChannelMgr;
use crate::net::tcp_client::TcpClientHandler;
use async_std::path;
use async_std::sync::Mutex;
use log::info;
use net::websocket;
use std::fs::{DirEntry, File};
use std::sync::Arc;
use tools::conf::Conf;

use crate::net::http::KickPlayerHttpHandler;
use crate::net::tcp_client::TcpClientType;
use crate::net::tcp_server;
use std::env;
use std::time::Duration;
use tools::http::HttpServerHandler;
use tools::redis_pool::RedisPoolTool;
use tools::tcp::ClientHandler;
use tools::thread_pool::MyThreadPool;

#[macro_use]
extern crate lazy_static;

//初始化全局线程池
lazy_static! {
    //初始化线程池
    static ref THREAD_POOL: MyThreadPool = {
        let game_name = "GAME_POOL".to_string();
        let user_name = "USER_POOL".to_string();
        let sys_name = "SYS_POOL".to_string();
        let mtp = MyThreadPool::init(game_name, 8, user_name, 8, sys_name, 4);
        mtp
    };
    static ref CONF_MAP: Conf = {
        let path = env::current_dir().unwrap();
        let str = path.as_os_str().to_str().unwrap();
        let res = str.to_string()+"/config/config.conf";
        let conf = Conf::init(res.as_str());
        conf
    };

   ///reids客户端
    static ref REDIS_POOL:Arc<std::sync::Mutex<RedisPoolTool>>={
        let add: &str = &CONF_MAP.get_str("redis_add","");
        let pass: &str = &CONF_MAP.get_str("redis_pass","");
        let redis = RedisPoolTool::init(add,pass);
        let redis:Arc<std::sync::Mutex<RedisPoolTool>> = Arc::new(std::sync::Mutex::new(redis));
        redis
    };
}

const REDIS_INDEX_USERS: u32 = 0;

const REDIS_KEY_USERS: &str = "users";

const REDIS_KEY_UID_2_PID: &str = "uid_2_pid";

fn main() {
    //创建核心结构体，channel管理器，因为涉及到多线程异步，所以创建结构体的arc引用计数器指针
    let cm = Arc::new(Mutex::new(ChannelMgr::new()));
    //初始化日志
    init_log();

    //连接游戏中心服
    init_game_center_tcp_connect(cm.clone());

    //连接游戏服务器
    init_game_tcp_connect(cm.clone());

    //初始化http服务
    init_http_server(cm.clone());

    //初始化与客户端通信的模块
    init_net_server(cm);
}

fn init_log() {
    let info_log = &CONF_MAP.get_str("info_log_path", "");
    let error_log = &CONF_MAP.get_str("error_log_path", "");
    tools::my_log::init_log(info_log, error_log);
}

///初始化http服务端
fn init_http_server(cm: Arc<Mutex<ChannelMgr>>) {
    std::thread::sleep(Duration::from_millis(10));
    let http_port = CONF_MAP.get_usize("http_port", 0);
    tools::http::Builder::new()
        .route(Box::new(KickPlayerHttpHandler::new(cm)))
        .bind(http_port as u16);
}

///初始化网络服务这块
fn init_net_server(cm: Arc<Mutex<ChannelMgr>>) {
    //获取通信模块
    let net_module = CONF_MAP.get_str("net_module", "");
    match net_module.as_str() {
        "tcp" => {
            //初始化tcp服务端
            init_tcp_server(cm);
        }
        "ws" => {
            init_ws_server(cm);
        }
        _ => {
            //初始化tcp服务端
            init_tcp_server(cm);
        }
    }
}

///初始化游戏服务器tcp客户端链接
fn init_game_tcp_connect(cp: Arc<Mutex<ChannelMgr>>) {
    let game = async {
        let mut tch = TcpClientHandler::new(cp, TcpClientType::GameServer);
        let address = CONF_MAP.get_str("game_port", "");
        info!("开始链接游戏服:{:?}...", address);
        tch.on_read(address.to_string()).await;
    };
    async_std::task::spawn(game);
}

///初始化房间服务器tcp客户端链接
fn init_game_center_tcp_connect(cp: Arc<Mutex<ChannelMgr>>) {
    let room = async {
        let mut tch = TcpClientHandler::new(cp, TcpClientType::GameCenter);
        let address = CONF_MAP.get_str("game_center_port", "");
        info!("开始链接游戏中心服:{:?}...", address);
        tch.on_read(address.to_string()).await;
    };
    async_std::task::spawn(room);
}

///初始化tcp服务端
fn init_tcp_server(cm: Arc<Mutex<ChannelMgr>>) {
    let str = &CONF_MAP.get_str("tcp_port", "");
    tcp_server::new(str, cm);
}

///初始化tcp服务端
fn init_ws_server(cm: Arc<Mutex<ChannelMgr>>) {
    let str = &CONF_MAP.get_str("web_socket_port", "");
    websocket::new(str, cm);
}
