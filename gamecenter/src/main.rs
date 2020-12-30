mod entity;
mod mgr;
mod net;

use crate::mgr::game_center_mgr::GameCenterMgr;
use async_std::sync::Mutex;
use std::env;
use std::sync::Arc;
use tools::conf::Conf;
use tools::my_log::init_log;

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
    let info_log = CONF_MAP.get_str("info_log_path");
    let error_log = CONF_MAP.get_str("error_log_path");
    //初始化日志模块
    init_log(info_log, error_log);
    let game_center = Arc::new(Mutex::new(GameCenterMgr::new()));
    init_tcp_server(game_center);
}

///初始化tcp服务端
fn init_tcp_server(rm: Arc<Mutex<GameCenterMgr>>) {
    let tcp_port: &str = CONF_MAP.get_str("tcp_port_gate");
    crate::net::gate_tcp_server::new(tcp_port, rm.clone());

    let tcp_port: &str = CONF_MAP.get_str("tcp_port_room");
    crate::net::battle_tcp_server::new(tcp_port, rm);
}
