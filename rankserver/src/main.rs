mod handler;
mod mgr;
mod net;
mod task_timer;

use std::env;

use crate::mgr::rank_mgr::RankMgr;
use crate::net::tcp_server;
use async_std::sync::Mutex;
use mgr::{RankInfo, RankInfoPtr};
use std::sync::Arc;
use task_timer::init_timer;
use tools::conf::Conf;
use tools::redis_pool::RedisPoolTool;
use tools::templates::template::{init_temps_mgr, TemplatesMgr};

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

    ///静态配置文件
    static ref TEMPLATES: TemplatesMgr = {
        init_templates_mgr()
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
type Lock = Arc<Mutex<RankMgr>>;

///排行榜redis索引
const REDIS_INDEX_RANK: u32 = 2;
///当前赛季排行
const REDIS_KEY_CURRENT_RANK: &str = "current_rank";

///上个赛季排行
const REDIS_KEY_LAST_RANK: &str = "last_rank";

///历史赛季排行
const REDIS_KEY_HISTORY_RANK: &str = "history_rank";

///最佳排行
const REDIS_KEY_BEST_RANK: &str = "best_rank";
fn main() {
    let rm = Arc::new(Mutex::new(RankMgr::new()));

    //初始化日志模块
    init_log();

    //初始化排行榜
    init_rank(rm.clone());

    //初始化定时器
    init_timer(rm.clone());

    //初始化网络
    init_tcp_server(rm.clone());
}

fn init_log() {
    let info_log = CONF_MAP.get_str("info_log_path");
    let error_log = CONF_MAP.get_str("error_log_path");
    tools::my_log::init_log(info_log, error_log);
}

///初始化tcp服务端
fn init_tcp_server(rm: Lock) {
    let tcp_port: &str = CONF_MAP.get_str("tcp_port");
    tcp_server::new(tcp_port, rm);
}

///初始化排行榜
fn init_rank(rm: Lock) {
    let mut redis_lock = async_std::task::block_on(REDIS_POOL.lock());
    let mut lock = async_std::task::block_on(rm.lock());

    //加载上一赛季排行榜
    let last_ranks: Option<Vec<String>> = redis_lock.hvals(REDIS_INDEX_RANK, REDIS_KEY_LAST_RANK);
    if let Some(last_ranks) = last_ranks {
        for last_rank in last_ranks {
            let ri: RankInfo = serde_json::from_str(last_rank.as_str()).unwrap();
            lock.last_rank.push(ri);
        }
    }
    //进行排序
    lock.last_rank.sort_unstable_by(|a, b| {
        //如果段位等级一样
        if a.league.get_league_id() == b.league.get_league_id() {
            if a.league.league_time != b.league.league_time {
                //看时间
                return a.league.league_time.cmp(&b.league.league_time);
            }
        }
        //段位不一样直接看分数
        b.get_score().cmp(&a.get_score())
    });

    //加载当前赛季排行榜
    let ranks: Option<Vec<String>> = redis_lock.hvals(REDIS_INDEX_RANK, REDIS_KEY_CURRENT_RANK);
    if ranks.is_none() {
        return;
    }
    let ranks = ranks.unwrap();

    for rank_str in ranks {
        let ri: RankInfo = serde_json::from_str(rank_str.as_str()).unwrap();
        let user_id = ri.user_id;
        lock.rank_vec.push(ri);
        let len = lock.rank_vec.len();
        let res = lock.rank_vec.get_mut(len - 1).unwrap();
        let res = RankInfoPtr(res as *mut RankInfo);
        lock.update_map.insert(user_id, res);
    }
    //进行排序
    lock.rank_vec.sort_unstable_by(|a, b| {
        //如果段位等级一样
        if a.league.get_league_id() == b.league.get_league_id() {
            if a.league.league_time != b.league.league_time {
                //看时间
                return a.league.league_time.cmp(&b.league.league_time);
            }
        }
        //段位不一样直接看分数
        b.get_score().cmp(&a.get_score())
    });
}
