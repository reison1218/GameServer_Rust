use log::{error, info};
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use tools::cmd_code::GameCode;

use crate::Lock;

#[derive(Default)]
pub struct Task {
    pub sql: String,
}

///初始化定时器任务函数
pub fn init_timer(rm: Lock) {
    let time = SystemTime::now();

    //每5分钟保存玩家数据
    sort_rank(rm.clone());

    info!(
        "定时任务初始化完毕!耗时:{:?}ms",
        time.elapsed().unwrap().as_millis()
    )
}

fn sort_rank(rm: Lock) {
    let mgr = crate::TEMPLATES.constant_temp_mgr();
    let update_time = mgr.temps.get("rank_update_time");
    let time;
    match update_time {
        Some(time_temp) => {
            let res = u64::from_str(time_temp.value.as_str());
            match res {
                Ok(time_res) => time = time_res,
                Err(e) => {
                    error!("{:?}", e);
                    time = 60 * 1000 * 10
                }
            }
        }
        None => time = 60 * 1000 * 10,
    }
    let m = move || {
        loop {
            std::thread::sleep(Duration::from_millis(time));
            let mut lock = async_std::task::block_on(rm.lock());
            if !lock.need_rank {
                info!("执行排行定时器-排行榜没有任何变化,无需排序");
                continue;
            }
            info!("执行排行定时器-开始执行排序");
            let take_time = std::time::SystemTime::now();
            //排序
            lock.sort(true);
            //设置不需要排序
            lock.need_rank = false;
            info!("执行排行定时器结束!耗时:{:?}", take_time.elapsed().unwrap());

            //下发到游戏服务器
            lock.push_2_server(GameCode::SyncRank.into_u32(), 0, vec![]);
            let take_time = std::time::SystemTime::now();
            let res = take_time.elapsed().unwrap();
            info!("更新rank并下发排行榜快照到游戏服结束!耗时{:?}", res);
        }
    };
    std::thread::spawn(m);
}
