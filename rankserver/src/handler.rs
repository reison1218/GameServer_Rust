use crate::mgr::RankInfo;
use crate::mgr::{rank_mgr::RankMgr, RankInfoPtr};
use crate::{
    REDIS_INDEX_RANK, REDIS_KEY_CURRENT_RANK, REDIS_KEY_HISTORY_RANK, REDIS_KEY_LAST_RANK,
    REDIS_POOL,
};
use async_std::task::block_on;
use log::{error, warn};
use protobuf::Message;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::str::FromStr;
use tools::cmd_code::GameCode;
use tools::protos::server_protocol::G_S_MODIFY_NICK_NAME;
use tools::protos::server_protocol::{
    B_S_SUMMARY, R_G_SYNC_RANK, R_G_UPDATE_LAST_SEASON_RANK, UPDATE_SEASON_NOTICE,
};
use tools::util::packet::Packet;

pub fn get_rank(rm: &mut RankMgr, packet: Packet) {
    let server_token = packet.get_server_token();
    let mut rgsr = R_G_SYNC_RANK::new();
    for ri in rm.rank_vec.iter() {
        let res = ri.into_rank_pt();
        rgsr.ranks.push(res);
    }
    let bytes = rgsr.write_to_bytes();
    if let Err(e) = bytes {
        error!("{:?}", e);
        return;
    }
    let bytes = bytes.unwrap();
    rm.send_2_server_direction(GameCode::SyncRank.into_u32(), 0, bytes, server_token);
}

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
    //通知所游戏服更新名字
    rm.push_2_server(
        GameCode::SyncRankNickName.into_u32(),
        0,
        packet.get_data().to_vec(),
    );
}

///处理上一赛季
///先清空上一赛季数据库，然后插入当前赛季数据
pub async fn process_last_season_rank(rm: &mut RankMgr, round: u32) {
    let mut redis_lock = REDIS_POOL.lock().await;

    //先清空上一赛季的排行榜数据
    let _: Option<String> = redis_lock.del(REDIS_INDEX_RANK, REDIS_KEY_LAST_RANK);

    //如果当前赛季到排行榜是空到，直接返回
    if rm.rank_vec.is_empty() {
        return;
    }

    //再插入新数据
    let mut proto = R_G_UPDATE_LAST_SEASON_RANK::new();
    let mut index = 0;
    rm.rank_vec.iter().for_each(|ri| {
        let user_id = ri.user_id;
        let json_value = serde_json::to_string(&ri).unwrap();
        let _: Option<String> = redis_lock.hset(
            REDIS_INDEX_RANK,
            REDIS_KEY_LAST_RANK,
            user_id.to_string().as_str(),
            json_value.as_str(),
        );
        if index < 100 {
            let key = format!("{:?}-,{:?}", REDIS_KEY_HISTORY_RANK, round.to_string());
            let _: Option<String> = redis_lock.hset(
                REDIS_INDEX_RANK,
                key.as_str(),
                user_id.to_string().as_str(),
                json_value.as_str(),
            );
        }

        proto.ranks.push(ri.into_rank_pt());
        index += 1;
    });

    let bytes = proto.write_to_bytes();
    if let Err(e) = bytes {
        error!("{:?}", e);
        return;
    }
    let bytes = bytes.unwrap();
    //通知所有游戏服更新上一赛季排行榜信息
    rm.push_2_server(GameCode::UpdateLastSeasonRankPush.into_u32(), 0, bytes);
}

pub fn update_season(rm: &mut RankMgr, packet: Packet) {
    let mut usn = UPDATE_SEASON_NOTICE::new();
    let res = usn.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let season_id = usn.get_season_id();
    let round = usn.get_round();

    let mgr = crate::TEMPLATES.constant_temp_mgr();
    let round_season_id = mgr.temps.get("round_season_id");
    if let None = round_season_id {
        warn!("the constant temp is None!key:round_season_id");
        return;
    }
    let round_season_id = round_season_id.unwrap();
    let res = u32::from_str(round_season_id.value.as_str());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let round_season_id = res.unwrap();
    if round_season_id != season_id {
        return;
    }

    //先处理上一赛季排行榜问题
    block_on(process_last_season_rank(rm, round));

    //处理当前赛季数据
    let mut remove_v = Vec::new();
    let mut redis_lock = async_std::task::block_on(REDIS_POOL.lock());

    //掉段处理
    rm.rank_vec.iter_mut().for_each(|x| {
        let user_id = x.user_id.to_string();
        x.league.id -= 1;
        let league_id = x.league.id;
        if x.league.id <= 0 {
            remove_v.push(x.user_id);
            redis_lock.hdel(REDIS_INDEX_RANK, REDIS_KEY_CURRENT_RANK, user_id.as_str());
        } else {
            x.update_league(league_id);
            let json_value = serde_json::to_string(x);
            match json_value {
                Ok(json_value) => {
                    let _: Option<String> = redis_lock.hset(
                        REDIS_INDEX_RANK,
                        REDIS_KEY_CURRENT_RANK,
                        user_id.as_str(),
                        json_value.as_str(),
                    );
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    });

    //清除0段位处理
    for user_id in remove_v {
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
    let res = rm.update_map.get_mut(&user_id);
    match res {
        Some(rank_ptr) => {
            rank_ptr.update(sd, cters);
        }
        None => {
            let ri = RankInfo::new(sd, cters);
            rm.rank_vec.push(ri);
            let len = rm.rank_vec.len();
            let ri_mut = rm.rank_vec.get_mut(len - 1).unwrap();
            rm.update_map
                .insert(user_id, RankInfoPtr(ri_mut as *mut RankInfo));
        }
    }
    rm.need_rank = true;
}
