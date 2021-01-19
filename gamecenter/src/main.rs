mod entity;
mod mgr;
mod net;

use crate::mgr::game_center_mgr::GameCenterMgr;
use crate::net::room_tcp_client::RoomTcpClientHandler;
use crate::net::{battle_tcp_server, gate_tcp_server};
use async_std::sync::Mutex;
use net::rank_tcp_client::RankTcpClientHandler;
use std::env;
use std::sync::Arc;
use tools::conf::Conf;
use tools::my_log::init_log;
use tools::tcp::ClientHandler;
use crate::net::http::{ReloadTempsHandler, UpdateSeasonHandler};
use tools::http::HttpServerHandler;
use std::time::Duration;

#[macro_use]
extern crate lazy_static;
//初始化全局线程池
lazy_static! {

    ///配置文件
    static ref CONF_MAP : Conf = {
        let path = env::current_dir().unwrap();
        let str = path.as_os_str().to_str().unwrap();
        let res = str.to_string()+"/config/config.conf";
        let conf = Conf::init(res.as_str());
        conf
    };
}

fn main() {
    let game_center = Arc::new(Mutex::new(GameCenterMgr::new()));
    let info_log = CONF_MAP.get_str("info_log_path");
    let error_log = CONF_MAP.get_str("error_log_path");
    //初始化日志模块
    init_log(info_log, error_log);
    //初始化tcp服务端
    init_tcp_server(game_center.clone());
    //初始化http服务器
    init_http_server(game_center.clone());
    //初始化tcp客户端
    init_tcp_client(game_center);
}

///初始化tcp服务端
fn init_tcp_server(gm: Arc<Mutex<GameCenterMgr>>) {
    let tcp_port: &str = CONF_MAP.get_str("tcp_port_gate");
    gate_tcp_server::new(tcp_port.to_string(), gm.clone());

    let tcp_port: &str = CONF_MAP.get_str("tcp_port_battle");
    battle_tcp_server::new(tcp_port.to_string(), gm);
}

///初始化tcp客户端
fn init_tcp_client(gm: Arc<Mutex<GameCenterMgr>>) {
    let mut rth = RoomTcpClientHandler { gm:gm.clone() };
    let address: &str = CONF_MAP.get_str("room_port");
    let m =  async move{
        rth.on_read(address.to_string()).await;
    };
    async_std::task::spawn(m);
    let address: &str = CONF_MAP.get_str("rank_port");
    let mut rth = RankTcpClientHandler{gm};
    let res = rth.on_read(address.to_string());
    async_std::task::block_on(res);
}

///初始化http服务端
fn init_http_server(gm: Arc<Mutex<GameCenterMgr>>) {
    std::thread::sleep(Duration::from_millis(10));
    let mut http_vec: Vec<Box<dyn HttpServerHandler>> = Vec::new();
    http_vec.push(Box::new(ReloadTempsHandler::new(gm.clone())));
    http_vec.push(Box::new(UpdateSeasonHandler::new(gm.clone())));
    let http_port: &str = CONF_MAP.get_str("http_port");
    async_std::task::spawn(tools::http::http_server(http_port, http_vec));
}
