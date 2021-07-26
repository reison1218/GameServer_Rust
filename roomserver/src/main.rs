mod handlers;
mod mgr;
mod net;
mod room;
mod task_timer;

use crate::mgr::room_mgr::RoomMgr;
use crate::net::tcp_server;
use crate::task_timer::init_timer;
use async_std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use log::{error, info, warn};
use scheduled_thread_pool::ScheduledThreadPool;
use serde_json::Value;
use std::env;
use std::sync::atomic::AtomicU32;
use tools::conf::Conf;
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
    static ref REDIS_POOL:Arc<std::sync::Mutex<RedisPoolTool>>={
        let add: &str = CONF_MAP.get_str("redis_add");
        let pass: &str = CONF_MAP.get_str("redis_pass");
        let redis = RedisPoolTool::init(add,pass);
        let redis:Arc<std::sync::Mutex<RedisPoolTool>> = Arc::new(std::sync::Mutex::new(redis));
        redis
    };
}

static ROBOT_ID: AtomicU32 = AtomicU32::new(0);

///赛季redis索引
const REDIS_INDEX_GAME_SEASON: u32 = 1;

///排行榜redis索引
const REDIS_INDEX_RANK: u32 = 2;

///赛季redis key
const REDIS_KEY_GAME_SEASON: &str = "game_season";

///当前赛季排行
const REDIS_KEY_CURRENT_RANK: &str = "current_rank";

///赛季数据
pub static mut SEASON: Season = Season::new();

pub struct Season {
    season_id: i32,
    next_update_time: u64,
}

impl Season {
    const fn new() -> Self {
        Season {
            season_id: 0,
            next_update_time: 0,
        }
    }
}

pub static mut ROOM_ID: Vec<u32> = Vec::new();

fn init_templates_mgr() -> TemplatesMgr {
    let path = env::current_dir().unwrap();
    let str = path.as_os_str().to_str().unwrap();
    let res = str.to_string() + "/template";
    let conf = init_temps_mgr(res.as_str());
    conf
}

fn init_room_id() {
    unsafe {
        for i in 100000..=999999 {
            ROOM_ID.push(i);
        }
    }
}

type Lock = Arc<Mutex<RoomMgr>>;

fn main() {
    //初始化room_mgr多线程饮用计数器指针
    let room_mgr = Arc::new(Mutex::new(RoomMgr::new()));

    //初始化日志模块
    init_log();

    //初始化配置
    init_temps();

    //初始化定时器任务
    init_timer(room_mgr.clone());

    //初始化赛季
    init_season();

    //初始化房间id
    init_room_id();

    //初始化tcp服务
    init_tcp_server(room_mgr.clone());
}

fn init_log() {
    let info_log = CONF_MAP.get_str("info_log_path");
    let error_log = CONF_MAP.get_str("error_log_path");
    tools::my_log::init_log(info_log, error_log);
}

///初始化赛季信息
fn init_season() {
    let mut lock = REDIS_POOL.lock().unwrap();
    unsafe {
        let res: Option<String> = lock.hget(REDIS_INDEX_GAME_SEASON, REDIS_KEY_GAME_SEASON, "101");
        let str = res.unwrap();
        let value = serde_json::from_str(str.as_str());
        if let Err(e) = value {
            error!("{:?}", e);
            return;
        }
        let value: Value = value.unwrap();
        let map = value.as_object();
        if map.is_none() {
            warn!("the map is None for JsonValue!");
            return;
        }
        let map = map.unwrap();

        let season_id = map.get("season_id");
        if season_id.is_none() {
            warn!("the season_id is None!");
            return;
        }
        let season_id = season_id.unwrap();
        let season_id = season_id.as_u64();
        if season_id.is_none() {
            warn!("the season_id is None!");
            return;
        }
        let season_id = season_id.unwrap();
        SEASON.season_id = season_id as i32;
        let next_update_time = map.get("next_update_time");
        if next_update_time.is_none() {
            warn!("the next_update_time is None!");
            return;
        }
        let next_update_time = next_update_time.unwrap();
        let next_update_time = next_update_time.as_u64();
        if next_update_time.is_none() {
            warn!("the next_update_time is None!");
            return;
        }
        let next_update_time = next_update_time.unwrap();
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
fn init_tcp_server(rm: Lock) {
    let tcp_port: &str = CONF_MAP.get_str("tcp_port");
    tcp_server::new(tcp_port, rm);
}
