use crate::mgr::rank_mgr::RankMgr;
use crate::mgr::RankInfo;
use crate::{
    REDIS_INDEX_HISTORY, REDIS_INDEX_RANK, REDIS_KEY_BEST_RANK, REDIS_KEY_CURRENT_RANK,
    REDIS_KEY_HISTORY_RANK, REDIS_KEY_LAST_RANK, REDIS_POOL,
};
use async_std::task::block_on;
use log::{error, warn};
use protobuf::Message;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::str::FromStr;
use tools::cmd_code::{BattleCode, GameCode, RoomCode};
use tools::protos::server_protocol::B_S_SUMMARY;
use tools::protos::server_protocol::G_S_MODIFY_NICK_NAME;
use tools::protos::server_protocol::R_S_UPDATE_SEASON;
use tools::util::packet::Packet;

///修改名字
pub fn modify_nick_name(rm: &mut RankMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let mut proto = G_S_MODIFY_NICK_NAME::new();
    let res = proto.merge_from_bytes(packet.get_data());
    if let Err(err) = res {
        error!("{:?}", err);
        return;
    }
    let nick_name = proto.nick_name;
    rm.rank_vec
        .par_iter_mut()
        .filter(|x| x.user_id == user_id)
        .for_each(|x| x.name = nick_name.clone());

    //同步最好排名
    let best_rank = rm.user_best_rank.get_mut(&user_id);
    if let Some(best_rank) = best_rank {
        best_rank.name = nick_name;
    }
    //通知所游戏服更新名字
    rm.push_2_server(
        GameCode::SyncRankNickName.into_u32(),
        0,
        packet.get_data().to_vec(),
    );
}

///处理上一赛季
///先清空上一赛季数据库，然后插入当前赛季数据
pub async fn handler_season_update(
    rm: &mut RankMgr,
    round: u32,
    round_season_id: i32,
    proto: &mut R_S_UPDATE_SEASON,
) {
    if round_season_id != proto.season_id {
        return;
    }

    let mut redis_lock = REDIS_POOL.lock().await;

    //先清空上一赛季的排行榜数据
    let _: Option<String> = redis_lock.del(REDIS_INDEX_RANK, REDIS_KEY_LAST_RANK);

    //如果当前赛季到排行榜是空到，直接返回
    if rm.rank_vec.is_empty() {
        return;
    }

    //立刻进行排序一次
    rm.sort(false);

    let mut index = 0;
    //刷新last_rank数据,并保存历史排行榜,并更新玩家历史最佳
    for ri in rm.rank_vec.iter() {
        let user_id = ri.user_id;
        let json_value = serde_json::to_string(ri);
        if let Err(err) = json_value {
            error!("{:?}", err);
            continue;
        }
        let json_value = json_value.unwrap();
        //更新last_rank
        let _: Option<String> = redis_lock.hset(
            REDIS_INDEX_RANK,
            REDIS_KEY_LAST_RANK,
            user_id.to_string().as_str(),
            json_value.as_str(),
        );
        //更新历史排行榜
        if index < 100 {
            let key = format!("{:?}-,{:?}", REDIS_KEY_HISTORY_RANK, round.to_string());
            let _: Option<String> = redis_lock.hset(
                REDIS_INDEX_HISTORY,
                key.as_str(),
                user_id.to_string().as_str(),
                json_value.as_str(),
            );
        }
        //更新玩家历史最佳
        let best_rank = rm.user_best_rank.get_mut(&user_id);
        let mut best_rank_temp = None;
        match best_rank {
            Some(best_rank) => {
                if best_rank.rank < ri.rank {
                    let _ = std::mem::replace(best_rank, ri.clone());
                    best_rank_temp = Some(best_rank.clone());
                }
            }
            None => {
                rm.user_best_rank.insert(user_id, ri.clone());
                best_rank_temp = Some(ri.clone());
            }
        }
        if let Some(best_rank_temp) = best_rank_temp {
            let best_rank_str = serde_json::to_string(&best_rank_temp);
            match best_rank_str {
                Ok(best_rank_str) => {
                    //更新到redis
                    let _: Option<String> = redis_lock.hset(
                        REDIS_INDEX_RANK,
                        REDIS_KEY_BEST_RANK,
                        user_id.to_string().as_str(),
                        best_rank_str.as_str(),
                    );
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
        index += 1;
    }

    //处理当前赛季数据
    let mut redis_lock = async_std::task::block_on(REDIS_POOL.lock());
    let mut need_rm_v = vec![];
    //掉段处理
    for ri in rm.rank_vec.iter_mut() {
        let user_id = ri.user_id;
        let user_id_str = user_id.to_string();

        ri.league.id -= 1;
        let league_id = ri.league.id;
        //清除0段位处理
        if ri.league.id <= 0 {
            need_rm_v.push(user_id);
            let _: Option<String> = redis_lock.hdel(
                REDIS_INDEX_RANK,
                REDIS_KEY_CURRENT_RANK,
                user_id_str.as_str(),
            );
        } else {
            ri.update_league(league_id);
            let json_value = serde_json::to_string(ri);
            match json_value {
                Ok(json_value) => {
                    let _: Option<String> = redis_lock.hset(
                        REDIS_INDEX_RANK,
                        REDIS_KEY_CURRENT_RANK,
                        user_id_str.as_str(),
                        json_value.as_str(),
                    );
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    }
    for user_id in need_rm_v {
        let mut index = 0;
        for ri in rm.rank_vec.iter() {
            if ri.user_id == user_id {
                break;
            };
            index += 1;
        }
        rm.rank_vec.remove(index);
    }
}

pub fn update_season(rm: &mut RankMgr, packet: Packet) {
    let mut usn = R_S_UPDATE_SEASON::new();
    let res = usn.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let round = usn.get_round();

    let mgr = crate::TEMPLATES.constant_temp_mgr();
    let round_season_id = mgr.temps.get("round_season_id");
    if let None = round_season_id {
        warn!("the constant temp is None!key:round_season_id");
        return;
    }
    let round_season_id = round_season_id.unwrap();
    let res = i32::from_str(round_season_id.value.as_str());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let round_season_id = res.unwrap();

    //处理赛季更新
    block_on(handler_season_update(rm, round, round_season_id, &mut usn));

    let bytes = usn.write_to_bytes();
    if let Err(e) = bytes {
        error!("{:?}", e);
        return;
    }
    let bytes = bytes.unwrap();
    //通知其他服赛季更新
    rm.push_2_server(GameCode::UpdateSeasonPush.into_u32(), 0, bytes.clone());
    rm.push_2_server(RoomCode::UpdateSeasonPush.into_u32(), 0, bytes.clone());
    rm.push_2_server(BattleCode::UpdateSeasonPush.into_u32(), 0, bytes);
}

///更新排行榜请求指令
pub fn update_rank(rm: &mut RankMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let mut bss = B_S_SUMMARY::new();
    let res = bss.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let sd = bss.get_summary_data();
    let cters = bss.cters.clone();
    let res = rm.get_rank_mut(user_id);
    match res {
        Some(ri) => {
            ri.update(sd, cters);
        }
        None => {
            let ri = RankInfo::new(sd, cters);
            rm.rank_vec.push(ri);
        }
    }
    rm.need_rank = true;
}
