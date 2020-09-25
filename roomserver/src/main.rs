mod battle;
mod handlers;
mod mgr;
mod net;
mod room;
mod task_timer;
#[macro_use]
extern crate lazy_static;

use crate::mgr::room_mgr::RoomMgr;
use crate::net::tcp_server;
use crate::task_timer::init_timer;
use log::{error, info};
use scheduled_thread_pool::ScheduledThreadPool;
use serde_json::Value;
use std::env;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
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

    ///reids客户端
    static ref REDIS_POOL:Arc<Mutex<RedisPoolTool>>={
        let add: &str = CONF_MAP.get_str("redis_add");
        let pass: &str = CONF_MAP.get_str("redis_pass");
        let redis = RedisPoolTool::init(add,pass);
        let redis:Arc<Mutex<RedisPoolTool>> = Arc::new(Mutex::new(redis));
        redis
    };
}

///赛季结构体
#[derive(Default)]
pub struct Season {
    season_id: u32,
    last_update_time: u64,
    next_update_time: u64,
}

///赛季信息
pub static mut SEASON: Season = new_season();

pub const fn new_season() -> Season {
    let res = Season {
        season_id: 0,
        last_update_time: 0,
        next_update_time: 0,
    };
    res
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
    let room_mgr = Arc::new(Mutex::new(RoomMgr::new()));
    //初始化定时器任务
    init_timer(room_mgr.clone());
    //初始化赛季
    init_season();
    //初始化tcp服务
    init_tcp_server(room_mgr.clone());
}

///初始化赛季信息
fn init_season() {
    let mut lock = REDIS_POOL.lock().unwrap();
    unsafe {
        let res: Option<String> = lock.hget(3, "game_season", "101");
        if let None = res {
            error!("redis do not has season data about game:{}", 101);
            return;
        }
        let str = res.unwrap();
        let value = Value::from(str);
        let map = value.as_object();
        if let None = map {
            return;
        }
        let map = map.unwrap();
        let season_id = map.get("season_id").unwrap().as_u64().unwrap() as u32;
        let last_update_time = map.get("last_update_time").unwrap().as_str().unwrap();
        let next_update_time = map.get("next_update_time").unwrap().as_str().unwrap();

        let last_update_time = chrono::NaiveDateTime::from_str(last_update_time)
            .unwrap()
            .timestamp() as u64;
        let next_update_time = chrono::NaiveDateTime::from_str(next_update_time)
            .unwrap()
            .timestamp() as u64;
        SEASON.season_id = season_id;
        SEASON.last_update_time = last_update_time;
        SEASON.next_update_time = next_update_time;
    }
}

fn init_temps() {
    let time = std::time::SystemTime::now();
    lazy_static::initialize(&TEMPLATES);
    let spend_time = time.elapsed().unwrap().as_millis();
    info!("初始化templates成功!耗时:{}ms", spend_time);
}

///初始化tcp服务端
fn init_tcp_server(rm: Arc<Mutex<RoomMgr>>) {
    let tcp_port: &str = CONF_MAP.get_str("tcp_port");
    tcp_server::new(tcp_port, rm);
}
