use crate::mgr::rank_mgr::RankMgr;
use crate::mgr::{RankInfo, RankInfoPtr};
use async_std::sync::Mutex;
use futures::executor::block_on;
use log::error;
use std::sync::Arc;
pub mod dbtool;

pub fn init_rank(rm: Arc<Mutex<RankMgr>>) {
    let sql = "select * from t_u_league where ";
    let res = crate::DB_POOL.exe_sql(sql, None);
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let q = res.unwrap();
    let mut v = Vec::new();
    for qr in q {
        let (_, data): (u32, serde_json::Value) = mysql::from_row(qr.unwrap());
        let ri = RankInfo::init_from_json(data);
        if let Err(e) = ri {
            error!("{:?}", e);
            continue;
        }
        let ri = ri.unwrap();
        //过滤掉新号,新号rank初始化为-1
        if ri.rank < 0 {
            continue;
        }
        v.push(ri);
    }

    let mut lock = block_on(rm.lock());
    for ri in v {
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
