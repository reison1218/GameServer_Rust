mod entity;
mod mgr;
mod net;
use crate::mgr::channel_mgr::ChannelMgr;
use crate::net::tcp_client::TcpClientHandler;
use log::info;
use net::websocket::WebSocketHandler;
use std::sync::{Arc, Mutex, RwLock};
use tools::conf::Conf;
use ws::{Builder, Sender as WsSender, Settings};

use crate::net::http::{KickPlayerHttpHandler, ReloadTempsHandler};
use crate::net::tcp_client::TcpClientType;
use crate::net::tcp_server;
use std::env;
use tools::http::HttpServerHandler;
use tools::my_log::init_log;
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
    static ref REDIS_POOL:Arc<RwLock<RedisPoolTool>>={
        let add: &str = CONF_MAP.get_str("redis_add");
        let pass: &str = CONF_MAP.get_str("redis_pass");
        let redis = RedisPoolTool::init(add,pass);
        let redis:Arc<RwLock<RedisPoolTool>> = Arc::new(RwLock::new(redis));
        redis
    };
}

fn main() {
    //获得日志配置
    let info_log = CONF_MAP.get_str("info_log_path");
    let error_log = CONF_MAP.get_str("error_log_path");

    //初始化日志
    init_log(info_log, error_log);

    //创建核心结构体，channel管理器，因为涉及到多线程异步，所以创建结构体的arc引用计数器指针
    let cm = Arc::new(Mutex::new(ChannelMgr::new()));

    //连接游戏服务器
    init_game_tcp_connect(cm.clone());

    //连接房间服务器
    init_room_tcp_connect(cm.clone());

    //初始化http服务
    init_http_server(cm.clone());

    //初始化与客户端通信的模块
    init_net_server(cm);
}

///初始化http服务端
fn init_http_server(gm: Arc<Mutex<ChannelMgr>>) {
    let mut http_vec: Vec<Box<dyn HttpServerHandler>> = Vec::new();
    http_vec.push(Box::new(KickPlayerHttpHandler::new(gm.clone())));
    http_vec.push(Box::new(ReloadTempsHandler::new(gm.clone())));
    let http_port: &str = CONF_MAP.get_str("http_port");
    async_std::task::spawn(tools::http::http_server(http_port, http_vec));
}

///初始化网络服务这块
fn init_net_server(cm: Arc<Mutex<ChannelMgr>>) {
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
fn init_game_tcp_connect(cp: Arc<Mutex<ChannelMgr>>) {
    let game = async {
        let mut tch = TcpClientHandler::new(cp, TcpClientType::GameServer);
        let address = CONF_MAP.get_str("game_port");
        info!("开始链接游戏服:{:?}...", address);
        tch.on_read(address.to_string());
    };
    async_std::task::spawn(game);
}

///初始化房间服务器tcp客户端链接
fn init_room_tcp_connect(cp: Arc<Mutex<ChannelMgr>>) {
    let room = async {
        let mut tch = TcpClientHandler::new(cp, TcpClientType::RoomServer);
        let address = CONF_MAP.get_str("room_port");
        info!("开始链接房间服:{:?}...", address);
        tch.on_read(address.to_string());
    };
    async_std::task::spawn(room);
}

///初始化tcp服务端
fn init_tcp_server(cm: Arc<Mutex<ChannelMgr>>) {
    let str = CONF_MAP.get_str("tcp_port");
    tcp_server::new(str, cm);
}

///初始化websocket
fn init_web_socket(cp: Arc<Mutex<ChannelMgr>>) {
    let mut setting = Settings::default();
    setting.max_connections = 2048;
    //websocket队列大小
    setting.queue_size = setting.max_connections * 2;
    //是否组合数据包
    setting.tcp_nodelay = true;
    let server = Builder::new()
        .with_settings(setting)
        .build(|out| {
            let arc: Arc<WsSender> = Arc::new(out);
            WebSocketHandler {
                ws: arc,
                add: None,
                cm: cp.clone(),
            }
        })
        .unwrap();
    let str = CONF_MAP.get_str("web_socket_port");
    let _web_socket = server.listen(str).unwrap();
}
