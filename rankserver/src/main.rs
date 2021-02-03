mod db;
mod handler;
mod mgr;
mod net;
mod task_timer;

use std::env;

use crate::db::init_rank;
use crate::mgr::rank_mgr::RankMgr;
use crate::net::tcp_server;
use async_std::sync::Mutex;
use db::dbtool::DbPool;
use std::sync::Arc;
use task_timer::init_timer;
use tools::conf::Conf;
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

        ///数据库链接池
    static ref DB_POOL: DbPool = {
       let db_pool = DbPool::new();
            db_pool
    };

    ///静态配置文件
    static ref TEMPLATES: TemplatesMgr = {
        init_templates_mgr()
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
fn init_tcp_server(rm: Arc<Mutex<RankMgr>>) {
    let tcp_port: &str = CONF_MAP.get_str("tcp_port");
    tcp_server::new(tcp_port, rm);
}
