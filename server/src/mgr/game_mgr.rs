use crate::entity::user::UserData;
use crate::entity::user_info::{
    create_room, get_last_season_rank, join_room, modify_grade_frame_and_soul, punish_match,
    search_room, show_rank, sync_rank, update_season,
};
use crate::entity::{Entity, EntityData};
use chrono::Local;
use log::{error, info, warn};
use protobuf::Message;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;
use tools::cmd_code::{ClientCode, GameCode, ServerCommonCode};
use tools::net_message_io::NetHandler;
use tools::protos::base::{PlayerPt, PunishMatchPt, WorldBossPt};
use tools::protos::base::{RankInfoPt, SeasonPt};
use tools::protos::protocol::{C_SYNC_DATA, S_SYNC_DATA, S_USER_LOGIN};
use tools::protos::server_protocol::{B_S_SUMMARY, G_S_MODIFY_NICK_NAME, UPDATE_WORLD_BOSS_PUSH};
use tools::util::packet::Packet;
use tools::{cmd_code::RankCode, protos::base::LeaguePt};

use super::RoomType;
use crate::helper::RankInfo;
use rayon::prelude::ParallelSliceMut;

pub struct RankInfoPtPtr(pub *mut RankInfoPt);
unsafe impl Send for RankInfoPtPtr {}
unsafe impl Sync for RankInfoPtPtr {}
///gameMgr结构体
pub struct GameMgr {
    pub users: HashMap<u32, UserData>,            //玩家数据
    pub rank: Vec<RankInfoPt>,                    //排行榜快照，从排行榜服务器那边过来的
    pub last_season_rank: Vec<RankInfoPt>,        //上一赛季排行榜
    pub user_best_rank: HashMap<u32, RankInfoPt>, //玩家最佳排行
    net_handler: Option<NetHandler>,              //tcpchannel
    pub cmd_map: HashMap<u32, fn(&mut GameMgr, Packet), RandomState>, //命令管理
}

impl GameMgr {
    ///创建gamemgr结构体
    pub fn new() -> GameMgr {
        let users: HashMap<u32, UserData> = HashMap::new();
        let mut gm = GameMgr {
            users,
            net_handler: None,
            rank: Vec::new(),
            last_season_rank: Vec::new(),
            user_best_rank: HashMap::new(),
            cmd_map: HashMap::new(),
        };
        //初始化命令
        gm.cmd_init();
        gm
    }

    pub fn get_ri_ref(&self, user_id: u32) -> Option<&RankInfoPt> {
        for ri in self.rank.iter() {
            if ri.user_id != user_id {
                continue;
            }
            return Some(ri);
        }
        None
    }

    pub fn get_ri_mut(&mut self, user_id: u32) -> Option<&mut RankInfoPt> {
        for ri in self.rank.iter_mut() {
            if ri.user_id != user_id {
                continue;
            }
            return Some(ri);
        }
        None
    }

    pub fn update_user_league_id(&mut self, user_id: u32, league_pt: LeaguePt) {
        let rank_pt_ptr = self.get_ri_mut(user_id);
        if rank_pt_ptr.is_none() {
            return;
        }
        let rank_pt = rank_pt_ptr.unwrap();
        rank_pt.set_league(league_pt);
    }

    ///初始化排行榜
    pub fn init_rank(&mut self) {
        self.last_season_rank.clear();
        self.user_best_rank.clear();
        self.rank.clear();
        let mut redis_lock = crate::REDIS_POOL.lock().unwrap();
        //加载当前排行榜
        let ranks: Option<Vec<String>> =
            redis_lock.hvals(crate::REDIS_INDEX_RANK, crate::REDIS_KEY_CURRENT_RANK);
        if let Some(ranks) = ranks {
            for rank_str in ranks {
                let ri: RankInfo = serde_json::from_str(rank_str.as_str()).unwrap();
                let rank_pt = ri.into_rank_pt();
                self.rank.push(rank_pt);
            }
            self.rank.par_sort_unstable_by(|a, b| a.rank.cmp(&b.rank));
        }

        //加载上赛季排行榜
        let last_ranks: Option<Vec<String>> =
            redis_lock.hvals(crate::REDIS_INDEX_RANK, crate::REDIS_KEY_LAST_RANK);
        if let Some(last_ranks) = last_ranks {
            for last_rank_str in last_ranks {
                let ri: RankInfo = serde_json::from_str(last_rank_str.as_str()).unwrap();
                self.last_season_rank.push(ri.into_rank_pt());
            }
            self.last_season_rank
                .par_sort_unstable_by(|a, b| a.rank.cmp(&b.rank));
        }

        //加载玩家最佳排名
        let best_ranks: Option<Vec<String>> =
            redis_lock.hvals(crate::REDIS_INDEX_RANK, crate::REDIS_KEY_BEST_RANK);
        if let Some(best_ranks) = best_ranks {
            for best_rank_str in best_ranks {
                let ri: RankInfo = serde_json::from_str(best_rank_str.as_str()).unwrap();
                self.user_best_rank.insert(ri.user_id, ri.into_rank_pt());
            }
        }
    }

    pub fn set_net_handler(&mut self, net_handler: NetHandler) {
        self.net_handler = Some(net_handler);
    }

    pub fn get_net_handler(&self) -> &NetHandler {
        self.net_handler.as_ref().unwrap()
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, true);
        let tcp = self.get_net_handler();
        let endpoint = tcp.endpoint;
        tcp.node_handler.network().send(endpoint, bytes.as_slice());
    }

    pub fn send_2_server(&mut self, cmd: u32, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, false);
        let tcp = self.get_net_handler();
        let endpoint = tcp.endpoint;
        tcp.node_handler.network().send(endpoint, bytes.as_slice());
    }

    pub fn save_user_http(&mut self) {
        let time = std::time::SystemTime::now();
        let mut count: u32 = 0;
        for (_, v) in self.users.iter_mut() {
            if v.get_version() <= 0 {
                continue;
            }
            v.update();
            count += 1;
        }
        info!(
            "玩家数据保存结束，保存个数:{},耗时：{}ms",
            count,
            time.elapsed().unwrap().as_millis()
        );
    }

    ///保存玩家数据
    pub fn save_user(&mut self, sender: crossbeam::channel::Sender<Vec<Box<dyn EntityData>>>) {
        let time = std::time::SystemTime::now();
        let mut v: Vec<Box<dyn EntityData>> = Vec::new();
        for ud in self.users.values_mut() {
            if ud.get_version() <= 0 {
                continue;
            };
            //装玩家
            if ud.get_user_info_ref().get_version() > 0 {
                v.push(ud.get_user_info_ref().try_clone_for_db());
            }
            //装角色
            let c_v = ud.get_characters_mut_ref().get_need_update_array();
            for i in c_v {
                v.push(i);
            }
            //grade相框数据
            if ud.grade_frame.get_version() > 0 {
                v.push(ud.grade_frame.try_clone_for_db());
            }
            //soul头像数据
            if ud.soul.get_version() > 0 {
                v.push(ud.soul.try_clone_for_db());
            }
            //由于这里是深拷贝，所以在这里提前清空版本号，不然在接收方那边执行update，清空的版本号也是clone的
            ud.clear_version();
        }
        let count = v.len();
        if count > 0 {
            let res = sender.send(v);
            match res {
                Err(e) => {
                    error!("{:?}", e.to_string());
                }
                _ => {}
            }
        }
        info!(
            "开始执行定时保存玩家，发送数量:{},耗时:{}ms",
            count,
            time.elapsed().unwrap().as_millis()
        );
    }

    ///执行函数，通过packet拿到cmd，然后从cmdmap拿到函数指针调用
    pub fn invok(&mut self, packet: Packet) {
        let cmd = packet.get_cmd();
        let f = self.cmd_map.get_mut(&cmd);
        match f {
            Some(func) => func(self, packet),
            None => {
                error!("there is no cmd:{}", cmd);
            }
        }
    }

    ///命令初始化
    fn cmd_init(&mut self) {
        self.cmd_map
            .insert(ServerCommonCode::ReloadTemps.into_u32(), reload_temps);
        self.cmd_map
            .insert(GameCode::UnloadUser.into_u32(), off_line);
        self.cmd_map
            .insert(GameCode::CreateRoom.into_u32(), create_room);
        self.cmd_map
            .insert(GameCode::JoinRoom.into_u32(), join_room);
        self.cmd_map
            .insert(GameCode::SearchRoom.into(), search_room);
        self.cmd_map
            .insert(GameCode::SyncPunish.into_u32(), punish_match);
        self.cmd_map
            .insert(GameCode::SyncRank.into_u32(), sync_rank);
        self.cmd_map
            .insert(GameCode::ShowRank.into_u32(), show_rank);
        self.cmd_map.insert(
            GameCode::ModifyGradeFrameAndSoul.into_u32(),
            modify_grade_frame_and_soul,
        );
        self.cmd_map
            .insert(GameCode::UpdateSeasonPush.into_u32(), update_season);
        self.cmd_map
            .insert(GameCode::GetLastSeasonRank.into_u32(), get_last_season_rank);
        self.cmd_map.insert(GameCode::Summary.into_u32(), summary);
        self.cmd_map
            .insert(GameCode::SyncRankNickName.into_u32(), sync_rank_nick_name);
        self.cmd_map
            .insert(GameCode::UpdateWorldBossPush.into_u32(), update_worldboss);
    }

    ///user结构体转proto
    pub fn user2proto(&mut self, user_id: u32) -> S_USER_LOGIN {
        let mut lr = S_USER_LOGIN::new();
        lr.set_is_succ(true);
        // let result = user.get_json_value("signInTime");
        // if result.is_some() {
        //     let str = result.unwrap().as_str().unwrap();
        //     let mut sign_in_Time = str.parse::<NaiveDateTime>();
        //     lr.signInTime = sign_in_Time.unwrap().timestamp_subsec_micros();
        // }
        let user = self.users.get_mut(&user_id).unwrap();
        let last_login_time =
            chrono::NaiveDateTime::from_str(user.get_user_info_mut_ref().last_login_time.as_str());
        let last_logoff_time =
            chrono::NaiveDateTime::from_str(user.get_user_info_mut_ref().last_off_time.as_str());
        let best_rank = self.user_best_rank.get(&user_id);
        let user_info = user.get_user_info_ref();
        let mut time = user_info.sync_time;
        lr.sync_time = time;
        let mut ppt = PlayerPt::new();
        let nick_name = user_info.nick_name.as_str();
        ppt.set_nick_name(nick_name.to_string());
        let last_character = user_info.last_character;
        ppt.set_last_character(last_character);
        ppt.dlc.push(1);
        let punish_match_pt: PunishMatchPt = user_info.punish_match.into();
        ppt.set_punish_match(punish_match_pt);
        ppt.set_grade(user_info.grade);
        ppt.set_grade_frame(user_info.grade_frame);
        ppt.set_soul(user_info.soul);
        match best_rank {
            Some(best_rank) => {
                ppt.set_best_rank(best_rank.rank);
            }
            None => {
                ppt.set_best_rank(-1);
            }
        }

        for ri in self.rank.iter() {
            if ri.user_id != user_id {
                continue;
            }
            ppt.set_league(ri.get_league().clone());
            break;
        }

        lr.set_player_pt(ppt);

        //封装赛季和worldboss信息
        let mut season_pt = SeasonPt::new();
        let mut world_boss_pt = WorldBossPt::new();
        unsafe {
            season_pt.set_season_id(crate::SEASON.season_id as u32);
            season_pt.set_end_time(crate::SEASON.next_update_time * 1000);
            world_boss_pt.set_world_boss_id(crate::WORLD_BOSS.world_boss_id as u32);
            world_boss_pt.set_end_time(crate::WORLD_BOSS.next_update_time * 1000)
        }
        lr.set_season_pt(season_pt);
        lr.set_world_boss_pt(world_boss_pt);
        time = 0;

        if let Ok(res) = last_login_time {
            time = res.timestamp_subsec_micros();
        }
        lr.last_login_time = time;
        time = 0;

        if let Ok(res) = last_logoff_time {
            time = res.timestamp_subsec_micros();
        }

        lr.last_logoff_time = time;

        for cter in user.get_characters_ref().cter_map.values() {
            lr.cters.push(cter.clone().into())
        }

        //封装grade相框
        lr.grade_frames
            .extend_from_slice(user.grade_frame.grade_frames.as_slice());
        //封装soul头像
        lr.souls.extend_from_slice(user.soul.souls.as_slice());
        lr
    }
}

///热更新配置文件
pub fn reload_temps(_: &mut GameMgr, _: Packet) {
    let path = std::env::current_dir();
    if let Err(e) = path {
        warn!("{:?}", e);
        return;
    }
    let path = path.unwrap();
    let str = path.as_os_str().to_str();
    if let None = str {
        warn!("reload_temps can not path to_str!");
        return;
    }
    let str = str.unwrap();
    let res = str.to_string() + "/template";
    let res = crate::TEMPLATES.reload_temps(res.as_str());
    match res {
        Ok(_) => {
            info!("reload_temps success!");
        }
        Err(e) => {
            warn!("{:?}", e);
        }
    }
}

///同步数据
fn sync(gm: &mut GameMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let user = gm.users.get_mut(&user_id);
    if user.is_none() {
        let str = format!("user data is null for id:{}", user_id);
        error!("{:?}", str.as_str());
        return;
    }
    let user = user.unwrap();

    let mut csd = C_SYNC_DATA::new();
    let res = csd.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        let str = format!(
            "protobuf:C_SYNC_DATA parse has error!cmd:{},err_mess:{:?}",
            packet.get_cmd(),
            e.to_string()
        );
        error!("{:?}", str.as_str());
        return;
    }

    if csd.player_pt.is_some() {
        let pp = csd.player_pt.unwrap();
        let nick_name = user.get_user_info_mut_ref().get_nick_name();
        if pp.get_nick_name() != nick_name {
            user.get_user_info_mut_ref()
                .set_nick_name(pp.get_nick_name());
        }
        user.get_user_info_mut_ref().set_dlc(pp.dlc);
    }

    let mut s_s_d = S_SYNC_DATA::new();
    s_s_d.is_succ = true;
    s_s_d.sync_time = Local::now().timestamp() as u32;
    gm.send_2_client(
        ClientCode::SyncData,
        user_id,
        s_s_d.write_to_bytes().unwrap(),
    );
    info!("执行同步函数");
}

///玩家离线
fn off_line(gm: &mut GameMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let user = gm.users.remove(&user_id);
    if let Some(mut user_data) = user {
        user_data.update_off();
        info!("游戏服已处理玩家离线 for id:{}", user_id);
    }
}

pub fn sync_rank_nick_name(gm: &mut GameMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let mut proto = G_S_MODIFY_NICK_NAME::new();
    let res = proto.merge_from_bytes(packet.get_data());
    if let Err(err) = res {
        error!("{:?}", err);
        return;
    }
    let nick_name = proto.nick_name;
    let rank_pt = gm.get_ri_mut(user_id);
    if let Some(rank_pt) = rank_pt {
        rank_pt.set_name(nick_name);
    }
}

pub fn update_worldboss(_: &mut GameMgr, packet: Packet) {
    let mut proto = UPDATE_WORLD_BOSS_PUSH::new();
    let res = proto.merge_from_bytes(packet.get_data());
    if let Err(err) = res {
        error!("{:?}", err);
        return;
    }
    let world_boss_id = proto.world_boss_id;
    let next_update_time = proto.next_update_time;
    unsafe {
        crate::WORLD_BOSS.world_boss_id = world_boss_id;
        crate::WORLD_BOSS.next_update_time = next_update_time;
    }
}

///房间战斗结算
pub fn summary(gm: &mut GameMgr, packet: Packet) {
    let mut bgs = B_S_SUMMARY::new();
    let res = bgs.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let room_type = RoomType::try_from(bgs.room_type as u8);
    if let Err(e) = room_type {
        error!("{:?}", e);
        return;
    }
    let room_type = room_type.unwrap();
    let summary_data_pt = bgs.get_summary_data().clone();
    let user_id = summary_data_pt.user_id;
    let cter_id = summary_data_pt.cter_id;
    let grade = summary_data_pt.get_grade();
    let res = gm.users.get_mut(&user_id);
    if let None = res {
        error! {"summary!UserData is not find! user_id:{}",user_id};
        return;
    }
    let user_data = res.unwrap();
    //处理统计
    let cters = user_data.get_characters_mut_ref().add_use_times(cter_id);
    //处理持久化到数据库
    user_data.add_version();
    //如果是匹配房
    if room_type.is_match_type() {
        //第一名就加grade
        user_data.user_info.set_grade(grade);
        if room_type == RoomType::OneVOneVOneVOneMatch {
            bgs.cters.extend_from_slice(cters.as_slice());
            let league_pt = summary_data_pt.get_league();
            //更新段位积分
            gm.update_user_league_id(user_id, league_pt.clone());
            if league_pt.league_id > 0 {
                let res = bgs.write_to_bytes();
                match res {
                    Ok(bytes) => {
                        //更新排行榜
                        gm.send_2_server(RankCode::UpdateRank.into_u32(), user_id, bytes);
                    }
                    Err(e) => {
                        warn!("{:?}", e)
                    }
                }
            }
        }
    }
}
