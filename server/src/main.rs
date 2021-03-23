mod db;
mod entity;
mod helper;
mod mgr;
mod net;
use crate::db::dbtool::DbPool;
use crate::mgr::game_mgr::GameMgr;
use crate::net::http::{SavePlayerHttpHandler, StopServerHttpHandler};
use crate::net::tcp_server;
use entity::league::League;
use entity::Entity;
use mysql::QueryResult;
use rayon::slice::ParallelSliceMut;
use tools::protos::base::RankInfoPt;
use tools::thread_pool::MyThreadPool;

use async_std::sync::Mutex;
use std::sync::Arc;

use crate::mgr::timer_mgr::init_timer;
use log::{error, info, warn};
use std::env;
use tools::conf::Conf;
use tools::http::HttpServerHandler;
use tools::redis_pool::RedisPoolTool;
use tools::templates::template::{init_temps_mgr, TemplatesMgr};

#[macro_use]
extern crate lazy_static;

//初始化全局线程池
lazy_static! {

    ///线程池
    static ref THREAD_POOL: MyThreadPool = {
        let game_model = "game_model".to_string();
        let user_model = "user_model".to_string();
        let sys_model = "sys_model".to_string();
        let mtp = MyThreadPool::init(game_model, 8, user_model, 8, sys_model, 2);
        mtp
    };

    ///数据库链接池
    static ref DB_POOL: DbPool = {
        let db_pool = DbPool::new();
        db_pool
    };

    ///配置文件
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
        let conf = init_temps_mgr(res.as_str());
        conf
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
const REDIS_INDEX_USERS: u32 = 0;

const REDIS_KEY_USERS: &str = "users";

const REDIS_KEY_UID_2_PID: &str = "uid_2_pid";

const REDIS_KEY_NAME_2_UID: &str = "name_2_uid";

const REDIS_INDEX_GAME_SEASON: u32 = 1;

const REDIS_KEY_GAME_SEASON: &str = "game_season";

///赛季信息
pub static mut SEASON: Season = Season::new();

pub struct Season {
    season_id: u32,
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
type Lock = Arc<Mutex<GameMgr>>;
type JsonValue = serde_json::Value;
///程序主入口,主要作用是初始化日志，数据库连接，redis连接，线程池，websocket，http
fn main() {
    let game_mgr = Arc::new(Mutex::new(GameMgr::new()));

    //初始化日志模块
    init_log();

    //初始化配置
    init_temps();

    //初始化定时器任务管理
    init_timer(game_mgr.clone());

    //初始化赛季
    init_season(game_mgr.clone());

    //初始化http服务端
    init_http_server(game_mgr.clone());

    //初始化tcp服务端
    init_tcp_server(game_mgr.clone());
}

fn init_log() {
    let info_log = CONF_MAP.get_str("info_log_path");
    let error_log = CONF_MAP.get_str("error_log_path");
    tools::my_log::init_log(info_log, error_log);
}

///初始化赛季信息
fn init_season(gm: Lock) {
    query_last_season_rank(gm);
    let mut lock = REDIS_POOL.lock().unwrap();
    let res: Option<String> = lock.hget(REDIS_INDEX_GAME_SEASON, REDIS_KEY_GAME_SEASON, "101");
    if let None = res {
        error!("redis do not has season data about game:{}", 101);
        return;
    }
    let res: Option<String> = lock.hget(REDIS_INDEX_GAME_SEASON, REDIS_KEY_GAME_SEASON, "101");
    let str = res.unwrap();
    let value = serde_json::from_str(str.as_str());
    if let Err(e) = value {
        error!("{:?}", e);
        return;
    }
    let value: JsonValue = value.unwrap();
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
    unsafe {
        SEASON.season_id = season_id as u32;
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

///初始化http服务端
fn init_http_server(gm: Lock) {
    let mut http_vec: Vec<Box<dyn HttpServerHandler>> = Vec::new();
    http_vec.push(Box::new(SavePlayerHttpHandler::new(gm.clone())));
    http_vec.push(Box::new(StopServerHttpHandler::new(gm.clone())));
    let http_port: &str = CONF_MAP.get_str("http_port");
    async_std::task::spawn(tools::http::http_server(http_port, http_vec));
}

///init tcp server
fn init_tcp_server(gm: Lock) {
    let tcp_port: &str = CONF_MAP.get_str("tcp_port");
    tcp_server::new(tcp_port, gm);
}

pub fn query_last_season_rank(gm: Lock) {
    let mut sql = String::new();
    sql.push_str("select * from t_u_last_season_rank");

    let q: Result<QueryResult, mysql::error::Error> = DB_POOL.exe_sql(sql.as_str(), None);
    if q.is_err() {
        error!("{:?}", q.err().unwrap());
        return;
    }
    let q = q.unwrap();
    let mut v = Vec::new();
    for _qr in q {
        let (_, data): (u32, serde_json::Value) = mysql::from_row(_qr.unwrap());
        let c = League::init(data);
        v.push(c.into());
    }
    v.par_sort_by(|a: &RankInfoPt, b: &RankInfoPt| a.rank.cmp(&b.rank));
    let mut lock = async_std::task::block_on(gm.lock());
    lock.last_season_rank = v;
}
