use crate::mgr::rank_mgr::RankMgr;
use async_std::sync::Mutex;
use log::info;
use rayon::prelude::*;
use rayon::slice::ParallelSliceMut;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tools::protos::server_protocol::R_G_SYNC_RANK;

///初始化定时器任务函数
pub fn init_timer(rm: Arc<Mutex<RankMgr>>) {
    let time = SystemTime::now();
    //每5分钟保存玩家数据
    update_rank(rm.clone());
    info!(
        "定时任务初始化完毕!耗时:{:?}ms",
        time.elapsed().unwrap().as_millis()
    )
}

fn update_rank(rm: Arc<Mutex<RankMgr>>) {
    let m = async move {
        loop {
            async_std::task::sleep(Duration::from_secs(60 * 5)).await;
            let mut lock = rm.lock().await;
            if !lock.need_rank {
                info!("执行排行定时器-排行榜没有任何变化,无需排序");
                continue;
            }
            info!("执行排行定时器-开始执行排序");
            let take_time = std::time::SystemTime::now();
            lock.rank_vec.par_sort_unstable_by(|a, b| {
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
            lock.need_rank = false;
            info!(
                "执行排行定时器结束!-耗时:{:?}",
                take_time.elapsed().unwrap()
            );
            //todo 重新排行之后下发到游戏服
            info!("更新rank并下发排行榜快照到游戏服开始！");
            let take_time = std::time::SystemTime::now();
            let mut rgsr = R_G_SYNC_RANK::new();
            lock.rank_vec
                .par_iter_mut()
                .enumerate()
                .for_each(move |(index, ri)| {
                    let index = index as i32;
                    if ri.rank != index {
                        ri.rank = index as i32;
                        //todo 更新数据库
                    }
                    let res = ri.into_rank_pt();
                    rgsr.ranks.push(res);
                });
            //todo 下发到游戏服务器
            info!(
                "更新rank并下发排行榜快照到游戏服结束！耗时{:?}",
                take_time.elapsed().unwrap()
            );
        }
    };
    async_std::task::spawn(m);
}
