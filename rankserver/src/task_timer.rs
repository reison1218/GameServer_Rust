use crate::REDIS_INDEX_RANK;
use crate::REDIS_KEY_CURRENT_RANK;
use crate::REDIS_POOL;
use log::{error, info};
use protobuf::Message;
use rayon::slice::ParallelSliceMut;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use tools::cmd_code::GameCode;
use tools::protos::server_protocol::R_G_SYNC_RANK;

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
    let m = async move {
        let mut redis_lock = REDIS_POOL.lock().await;
        loop {
            async_std::task::sleep(Duration::from_millis(time)).await;
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
            info!("执行排行定时器结束!耗时:{:?}", take_time.elapsed().unwrap());
            //重新排行之后下发到游戏服
            let take_time = std::time::SystemTime::now();
            let mut rgsr = R_G_SYNC_RANK::new();

            lock.rank_vec
                .iter_mut()
                .enumerate()
                .for_each(|(index, ri)| {
                    if ri.rank != index as i32 && ri.league.id > 0 {
                        ri.rank = index as i32;
                        let user_id = ri.user_id;
                        let json_value = serde_json::to_string(ri).unwrap();
                        //持久化到redis
                        let _: Option<String> = redis_lock.hset(
                            REDIS_INDEX_RANK,
                            REDIS_KEY_CURRENT_RANK,
                            user_id.to_string().as_str(),
                            json_value.as_str(),
                        );
                    }
                    let res = ri.into_rank_pt();
                    rgsr.ranks.push(res);
                });
            let bytes = rgsr.write_to_bytes();
            if let Err(e) = bytes {
                error!("{:?}", e);
                continue;
            }
            let bytes = bytes.unwrap();
            //下发到游戏服务器
            lock.push_2_server(GameCode::SyncRank.into_u32(), 0, bytes);
            let res = take_time.elapsed().unwrap();
            info!("更新rank并下发排行榜快照到游戏服结束!耗时{:?}", res);
        }
    };
    async_std::task::spawn(m);
}
