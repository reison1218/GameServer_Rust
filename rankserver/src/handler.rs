use crate::mgr::RankInfo;
use crate::mgr::{rank_mgr::RankMgr, RankInfoPtr};
use crate::task_timer::Task;
use log::{error, warn};
use protobuf::Message;
use std::str::FromStr;
use tools::cmd_code::GameCode;
use tools::protos::server_protocol::{
    B_S_SUMMARY, R_G_SYNC_RANK, R_G_UPDATE_LAST_SEASON_RANK, UPDATE_SEASON_NOTICE,
};
use tools::util::packet::Packet;

pub fn get_rank(rm: &mut RankMgr, packet: Packet) -> anyhow::Result<()> {
    let server_token = packet.get_server_token();
    let mut rgsr = R_G_SYNC_RANK::new();
    for ri in rm.rank_vec.iter() {
        let res = ri.into_rank_pt();
        rgsr.ranks.push(res);
    }
    let bytes = rgsr.write_to_bytes();
    if let Err(e) = bytes {
        error!("{:?}", e);
        return Ok(());
    }
    let bytes = bytes.unwrap();
    rm.send_2_server_direction(GameCode::SyncRank.into_u32(), 0, bytes, server_token);
    Ok(())
}

///处理上一赛季
///先清空上一赛季数据库，然后插入当前赛季数据
pub fn process_last_season_rank(rm: &mut RankMgr) {
    //先清空上一赛季的排行榜数据
    let delete_sql = "delete from t_u_last_season_rank";
    let res = crate::DB_POOL.exe_sql(delete_sql, None);
    if let Err(e) = res {
        error!("{:?}", e);
    }
    let mut size = 99;
    let len = rm.rank_vec.len();
    if len < size {
        size = len - 1;
    }
    let res = &rm.rank_vec[0..size];
    //再插入新数据
    let mut proto = R_G_UPDATE_LAST_SEASON_RANK::new();
    for ri in res {
        let insert_sql = ri.get_insert_sql_str();
        let res = crate::DB_POOL.exe_sql(insert_sql.as_ref(), None);
        if let Err(e) = res {
            error!("{:?}", e)
        }
        proto.ranks.push(ri.into_rank_pt())
    }
    let bytes = proto.write_to_bytes();
    if let Err(e) = bytes {
        error!("{:?}", e);
        return;
    }
    let bytes = bytes.unwrap();
    //通知所有游戏服更新上一赛季排行榜信息
    rm.push_2_server(GameCode::UpdateLastSeasonRankPush.into_u32(), 0, bytes);
}

pub fn update_season(rm: &mut RankMgr, packet: Packet) -> anyhow::Result<()> {
    let mut usn = UPDATE_SEASON_NOTICE::new();
    let res = usn.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let season_id = usn.get_season_id();

    let mgr = crate::TEMPLATES.get_constant_temp_mgr_ref();
    let round_season_id = mgr.temps.get("round_season_id");
    if let None = round_season_id {
        warn!("the constant temp is None!key:round_season_id");
        return Ok(());
    }
    let round_season_id = round_season_id.unwrap();
    let res = u32::from_str(round_season_id.value.as_str());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let round_season_id = res.unwrap();
    if round_season_id != season_id {
        return Ok(());
    }

    //先处理上一赛季排行榜问题
    process_last_season_rank(rm);

    //处理当前赛季数据
    let task_task = rm.task_sender.clone().unwrap();
    let mut remove_v = Vec::new();
    //掉段处理
    rm.rank_vec.iter_mut().for_each(|x| {
        x.league.id -= 1;
        let league_id = x.league.id;
        let sql_res;
        let mut task = Task::default();
        if x.league.id <= 0 {
            x.reset();
            sql_res = format!(r#"update t_u_league set content = JSON_SET(content, "$.rank", -1),content=JSON_SET(content,"&.score",0),content=JSON_SET(content,"$.league_time",0),content=JSON_SET(content,"$.id",0) where user_id = {}"#,x.user_id);
            remove_v.push(x.user_id);
        }else {
            x.update_league(league_id);
            sql_res = format!(r#"update t_u_league set content=JSON_SET(content,"&.score",{}),content=JSON_SET(content,"$.league_time",{}),content=JSON_SET(content,"$.id",{}) where user_id = {}"#,x.league.league_score,x.league.league_time,x.league.id,x.user_id);
        }
        task.sql = sql_res;
        let _=task_task.send(task);
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
    Ok(())
}

///更新排行榜请求指令
pub fn update_rank(rm: &mut RankMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut bss = B_S_SUMMARY::new();
    let res = bss.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
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
    Ok(())
}
