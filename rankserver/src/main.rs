mod handler;
mod mgr;
mod net;
mod task_timer;

use std::{env, time::Duration};

use crate::mgr::rank_mgr::RankMgr;
use crate::net::tcp_server;
use async_std::sync::Mutex;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::Arc;
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
    let mut rm = RankMgr::new();
    rm.update_rank_info();
    println!("{:?}", rm.rank_vec);
    std::thread::sleep(Duration::from_secs(5));
    // let rm = Arc::new(Mutex::new(RankMgr::new()));
    // init_tcp_server(rm.clone());
}

///初始化tcp服务端
fn init_tcp_server(rm: Arc<Mutex<RankMgr>>) {
    let tcp_port: &str = CONF_MAP.get_str("tcp_port");
    tcp_server::new(tcp_port, rm);
}
