mod entity;
mod mgr;
mod net;

#[macro_use]
extern crate lazy_static;

use crate::entity::room::Room;
use crate::mgr::room_mgr::RoomMgr;
use crate::net::tcp_server;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::sync::RwLock;
use tools::conf::Conf;
use tools::my_log::init_log;

///初始化全局线程池
lazy_static! {

    static ref CONF_MAP: Conf = {
        //let conf = Conf::init("/Users/tangjian/git/MyRust/roomserver/configs/config.conf");
        let conf = Conf::init("/game/game_server/room_server/config/config.conf");
        conf
    };
}

///全局静态变量，用来初始化房间id
pub static ROOM_ID: AtomicU64 = AtomicU64::new(101);

fn main() {
    let info_log = CONF_MAP.get_str("infoLogPath");
    let error_log = CONF_MAP.get_str("errorLogPath");
    //初始化日志模块
    init_log(info_log, error_log);

    let mut room_mgr: Arc<RwLock<RoomMgr>> = Arc::new(RwLock::new(RoomMgr::new()));
    init_tcp_server(room_mgr);
}

///初始化tcp服务端
fn init_tcp_server(rm: Arc<RwLock<RoomMgr>>) {
    let tcpPort: &str = CONF_MAP.get_str("tcpPort");
    tcp_server::new(tcpPort, rm);
}
