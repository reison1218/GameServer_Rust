use crate::mgr::rank_mgr::RankMgr;
use async_std::sync::Mutex;
use log::{error, info};
use protobuf::Message;
use rayon::prelude::*;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tools::cmd_code::GameCode;
use tools::protos::server_protocol::R_G_SYNC_RANK;

#[derive(Default)]
pub struct Task {
    pub sql: String,
}

///初始化定时器任务函数
pub fn init_timer(rm: Arc<Mutex<RankMgr>>) {
    let time = SystemTime::now();

    //每5分钟保存玩家数据
    sort_rank(rm.clone());

    //定时更新排行到数据库
    update_db(rm.clone());

    info!(
        "定时任务初始化完毕!耗时:{:?}ms",
        time.elapsed().unwrap().as_millis()
    )
}

fn update_db(rm: Arc<Mutex<RankMgr>>) {
    let (sender, receive) = crossbeam::channel::bounded(128);
    let mut lock = async_std::task::block_on(rm.lock());
    lock.set_task_sender(sender.clone());
    std::mem::drop(lock);
    let task_v = Arc::new(Mutex::new(Vec::new()));
    let task_v_clone = task_v.clone();
    let m = async move {
        loop {
            let task = receive.recv();
            if let Err(e) = task {
                error!("{:?}", e);
                continue;
            }
            let task = task.unwrap();
            let sql = task.sql.clone();
            let mut lock = task_v_clone.lock().await;
            lock.push(sql);
        }
    };
    async_std::task::spawn(m);
    let task_v_clone = task_v.clone();
    let m = async move {
        loop {
            async_std::task::sleep(Duration::from_millis(60 * 5)).await;
            let mut lock = task_v_clone.lock().await;
            for sql in lock.iter() {
                let res = crate::DB_POOL.exe_sql(sql.as_str(), None);
                if let Err(e) = res {
                    error!("{:?}", e);
                }
            }
            lock.clear();
        }
    };
    async_std::task::spawn(m);
}

fn sort_rank(rm: Arc<Mutex<RankMgr>>) {
    let mgr = crate::TEMPLATES.get_constant_temp_mgr_ref();
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
            info!(
                "执行排行定时器结束!-耗时:{:?}",
                take_time.elapsed().unwrap()
            );
            //重新排行之后下发到游戏服
            info!("更新rank并下发排行榜快照到游戏服开始！");
            let take_time = std::time::SystemTime::now();
            let mut rgsr = R_G_SYNC_RANK::new();
            let sender = lock.task_sender.clone().unwrap();
            lock.rank_vec
                .iter_mut()
                .enumerate()
                .for_each(|(index, ri)| {
                    if ri.rank != index as i32 {
                        ri.rank = index as i32;
                        //todo 更新数据库
                        let sql = format!(
                            r#"update t_u_league set content = JSON_SET(content, "$.rank", {}) where user_id = {}"#,
                            ri.rank, ri.user_id
                        );
                        let mut task = Task::default();
                        task.sql = sql;
                        let res = sender.send(task);
                        if let Err(e) = res{
                            error!("{:?}", e);
                        }
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
            lock.send_2_server(GameCode::SyncRank.into_u32(), 0, bytes);
            info!(
                "更新rank并下发排行榜快照到游戏服结束！耗时{:?}",
                take_time.elapsed().unwrap()
            );
        }
    };
    async_std::task::spawn(m);
}
