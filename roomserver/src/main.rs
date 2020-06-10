mod entity;
mod mgr;
mod net;
#[macro_use]
extern crate lazy_static;

use crate::mgr::room_mgr::RoomMgr;
use crate::net::tcp_server;
use std::env;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::sync::RwLock;
use tools::conf::Conf;
use tools::my_log::init_log;
use tools::templates::template::{init_temps, TemplatesMgr};

//初始化全局线程池
lazy_static! {

    static ref CONF_MAP: Conf = {
        let path = env::current_dir().unwrap();
        let str = path.as_os_str().to_str().unwrap();
        let res = str.to_string()+"/config/config.conf";
        let conf = Conf::init(res.as_str());
        conf
    };
    ///静态配置文件
    static ref TEMPLATES: TemplatesMgr = {
        let path = env::current_dir().unwrap();
        let str = path.as_os_str().to_str().unwrap();
        let res = str.to_string()+"/template";
        let conf = init_temps(res.as_str());
        conf
    };
}

///全局静态变量，用来初始化房间id
pub static ROOM_ID: AtomicU32 = AtomicU32::new(101);

fn main() {
    let info_log = CONF_MAP.get_str("info_log_path");
    let error_log = CONF_MAP.get_str("error_log_path");
    //初始化日志模块
    init_log(info_log, error_log);
    //初始化room_mgr多线程饮用计数器指针
    let room_mgr: Arc<RwLock<RoomMgr>> = Arc::new(RwLock::new(RoomMgr::new()));
    //初始化tcp服务
    init_tcp_server(room_mgr.clone());
}

///初始化tcp服务端
fn init_tcp_server(rm: Arc<RwLock<RoomMgr>>) {
    let tcp_port: &str = CONF_MAP.get_str("tcp_port");
    tcp_server::new(tcp_port, rm);
}
