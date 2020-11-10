mod handlers;
mod mgr;
mod net;
#[macro_use]
extern crate lazy_static;

use crate::mgr::room_mgr::RoomMgr;
use crate::net::tcp_server;
use async_std::sync::{Arc, RwLock};
use log::info;
use scheduled_thread_pool::ScheduledThreadPool;
use std::env;
use std::sync::atomic::AtomicU32;
use std::sync::Mutex;
use tools::conf::Conf;
use tools::my_log::init_log;
use tools::redis_pool::RedisPoolTool;
use tools::templates::template::{init_temps_mgr, TemplatesMgr};

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
    ///静态配置文件
    static ref TEMPLATES: TemplatesMgr = {
        init_templates_mgr()
    };

    ///定时器任务队列
    static ref SCHEDULED_MGR : ScheduledThreadPool = {
        let stp = ScheduledThreadPool::with_name("TASK_TIMER",8);
        stp
    };

    ///机器人定时器任务队列
    static ref ROBOT_SCHEDULED_MGR : ScheduledThreadPool = {
        let stp = ScheduledThreadPool::with_name("ROBOT_TASK_TIMER",8);
        stp
    };

    ///reids客户端
    static ref REDIS_POOL:Arc<Mutex<RedisPoolTool>>={
        let add: &str = CONF_MAP.get_str("redis_add");
        let pass: &str = CONF_MAP.get_str("redis_pass");
        let redis = RedisPoolTool::init(add,pass);
        let redis:Arc<Mutex<RedisPoolTool>> = Arc::new(Mutex::new(redis));
        redis
    };
}

fn init_templates_mgr() -> TemplatesMgr {
    let path = env::current_dir().unwrap();
    let str = path.as_os_str().to_str().unwrap();
    let res = str.to_string() + "/template";
    let conf = init_temps_mgr(res.as_str());
    conf
}

fn main() {
    let info_log = CONF_MAP.get_str("info_log_path");
    let error_log = CONF_MAP.get_str("error_log_path");
    //初始化日志模块
    init_log(info_log, error_log);
    //初始化配置
    init_temps();
    //初始化room_mgr多线程饮用计数器指针
    let room_mgr = Arc::new(RwLock::new(RoomMgr::new()));
    //初始化tcp服务
    init_tcp_server(room_mgr.clone());
}

fn init_temps() {
    let time = std::time::SystemTime::now();
    lazy_static::initialize(&TEMPLATES);
    let spend_time = time.elapsed().unwrap().as_millis();
    info!("初始化templates成功!耗时:{}ms", spend_time);
}

///初始化tcp服务端
fn init_tcp_server(rm: Arc<RwLock<RoomMgr>>) {
    let tcp_port: &str = CONF_MAP.get_str("tcp_port");
    tcp_server::new(tcp_port, rm);
}
