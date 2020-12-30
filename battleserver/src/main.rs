mod battle;
mod handlers;
mod mgr;
mod net;
mod robot;
mod room;
mod task_timer;

use scheduled_thread_pool::ScheduledThreadPool;
use std::env;
use tools::conf::Conf;
use tools::templates::template::{init_temps_mgr, TemplatesMgr};
use crate::task_timer::init_timer;
use crate::mgr::battle_mgr::BattleMgr;
use std::sync::Arc;
use async_std::sync::Mutex;

#[macro_use]
extern crate lazy_static;

//初始化全局线程池
lazy_static! {

    ///定时器任务队列
    static ref SCHEDULED_MGR : ScheduledThreadPool = {
        let stp = ScheduledThreadPool::with_name("TASK_TIMER",8);
        stp
    };

    ///静态配置文件
    static ref TEMPLATES: TemplatesMgr = {
        init_templates_mgr()
    };

    ///机器人定时器任务队列
    static ref ROBOT_SCHEDULED_MGR : ScheduledThreadPool = {
        let stp = ScheduledThreadPool::with_name("ROBOT_TASK_TIMER",8);
        stp
    };

    ///配置文件
    static ref CONF_MAP: Conf = {
        let path = env::current_dir().unwrap();
        let str = path.as_os_str().to_str().unwrap();
        let res = str.to_string()+"/config/config.conf";
        let conf = Conf::init(res.as_str());
        conf
    };
}

///赛季结构体
#[derive(Default, Debug)]
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
    let bm = Arc::new(Mutex::new(BattleMgr::new()));
    init_timer(bm.clone());
}
