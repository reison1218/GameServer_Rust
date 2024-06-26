pub mod battle;
pub mod fsm;
pub mod goal_ai;
pub mod handlers;
pub mod mgr;
pub mod net;

use crate::battle::battle::RobotCter;
use crate::fsm::miner::{Miner, Robot};
use crate::fsm::status::{EnterMineAndDigForNugget, Status};
use crate::goal_ai::cter::Cter;
use crate::goal_ai::goal_think::GoalThink;
use crate::mgr::robot_mgr::RobotMgr;
use crate::net::tcp_server::TcpServerHandler;
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
    //test_goal();
    // test_fsm();
    let info_log = &CONF_MAP.get_str("info_log_path","");
    let error_log = &CONF_MAP.get_str("error_log_path","");
    //初始化日志
    init_log(info_log, error_log);
    let rm = Arc::new(Mutex::new(RobotMgr::new()));
    ///初始化机器人服务器网络
    init_tcp_server(rm.clone());
}

fn test_fsm() {
    //let m = move || {
    let e = EnterMineAndDigForNugget {
        status: Status::EnterMineAndDigForNugget,
    };
    let status = Box::new(e);
    let mut miner = Miner::new(1, status);
    miner.update();
    // };
    // std::thread::spawn(m);
}

fn test_goal() {
    let m = move || {
        let mut cter = Cter::default();
        let mut gt = GoalThink::new();
        cter.goal_think = gt;
        cter.update();
    };
    std::thread::spawn(m);
}

///初始化tcp服务端
fn init_tcp_server(rm: Arc<Mutex<RobotMgr>>) {
    let sh = TcpServerHandler { sender: None, rm };
    let tcp_port = CONF_MAP.get_str("tcp_port","");
    
    async_std::task::block_on(async{
        let res = tcp_server::new(tcp_port, sh).await;
        if let Err(e) = res {
            error!("{:?}", e);
            std::process::abort();
        } 
    });
    
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
