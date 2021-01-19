use crate::mgr::RankInfo;
use crate::mgr::{rank_mgr::RankMgr, RankInfoPtr};
use crate::task_timer::Task;
use log::{error, warn};
use protobuf::Message;
use std::str::FromStr;
use tools::protos::server_protocol::{B_S_SUMMARY, UPDATE_SEASON_NOTICE};
use tools::util::packet::Packet;

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
    let task_task = rm.task_sender.clone().unwrap();
    let mut remove_v = Vec::new();
    //掉段处理
    rm.rank_vec.iter_mut().for_each(|x| {
        let old_id = x.league.id;
        x.league.id -= 1;
        if x.league.id <= 0 {
            x.league.id = 0;
            x.rank = -1;
            x.league.league_time = 0;
            let mut task = Task::default();
            let res = format!(r#"update t_u_league set content = JSON_SET(content, "$.rank", -1,"$.score",0,"$.id",0,"$.league_time",'') where user_id = {}"#,x.user_id);
            task.sql = res;
            let _=task_task.send(task);
            remove_v.push(x.user_id);
        }else {
            let res = crate::TEMPLATES
                .get_league_temp_mgr_ref()
                .get_temp(&x.league.id)
                .unwrap();
            if old_id != x.league.id {
                let time = chrono::Local::now();
                x.league.league_score = res.score;
                x.league.league_time = time.timestamp_millis();
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
    Ok(())
}

///更新赛季
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
