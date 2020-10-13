pub mod mgr;
pub mod net;
pub mod robot;

use crate::mgr::robot_mgr::RobotMgr;
use crate::net::tcp_server::TcpServerHandler;
use crate::robot::miner::{Miner, Robot};
use crate::robot::status::{EnterMineAndDigForNugget, Status};
use log::{error, info, LevelFilter};
use simplelog::{CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::{env, time};
use tools::conf::Conf;
use tools::tcp::tcp_server;

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
    test_robot();
    let info_log = CONF_MAP.get_str("info_log_path");
    let error_log = CONF_MAP.get_str("error_log_path");
    //初始化日志
    init_log(info_log, error_log);
    ///初始化机器人服务器网络
    init_tcp_server();
}

fn test_robot() {
    let m = move || {
        let mut miner = Miner::new(1);
        let e = EnterMineAndDigForNugget {
            status: Status::EnterMineAndDigForNugget,
            target: None,
        };
        miner.change_status(Box::new(e));
    };
    std::thread::spawn(m);
}

///初始化tcp服务端
fn init_tcp_server() {
    let sh = TcpServerHandler {
        sender: None,
        rm: Arc::new(Mutex::new(RobotMgr::default())),
    };
    let tcp_port: &str = CONF_MAP.get_str("tcp_port");
    let res = tcp_server::new(tcp_port, sh);
    if let Err(e) = res {
        error!("{:?}", e);
        std::process::abort();
    }
}

///初始化日志
/// 传入info_path作为 info文件路径
/// 传入error_path作为 error文件路径
pub fn init_log(info_path: &str, error_path: &str) {
    let log_time = time::SystemTime::now();
    let mut config = simplelog::ConfigBuilder::new();
    config.set_time_format_str("%Y-%m-%d %H:%M:%S");
    config.set_time_to_local(true);
    config.set_target_level(LevelFilter::Error);
    config.set_location_level(LevelFilter::Error);
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, config.build(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            LevelFilter::Info,
            config.build(),
            File::create(info_path).unwrap(),
        ),
        WriteLogger::new(
            LevelFilter::Error,
            config.build(),
            File::create(error_path).unwrap(),
        ),
    ])
    .unwrap();
    info!(
        "日志模块初始化完成！耗时:{}ms",
        log_time.elapsed().unwrap().as_millis()
    );
}
