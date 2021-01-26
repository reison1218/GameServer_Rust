use super::*;
use crate::helper::redis_helper::modify_redis_user;
use crate::mgr::RoomType;
use crate::mgr::game_mgr::RankInfoPtPtr;
use chrono::Local;
use protobuf::Message;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::convert::TryFrom;
use std::str::FromStr;
use tools::cmd_code::{ClientCode, RankCode, RoomCode};
use tools::protos::base::{PunishMatchPt, RankInfoPt};
use tools::protos::protocol::{
    C_MODIFY_GRADE_FRAME_AND_SOUL, C_MODIFY_NICK_NAME, S_MODIFY_GRADE_FRAME_AND_SOUL,
    S_MODIFY_NICK_NAME, S_SHOW_RANK,
};
use tools::protos::room::{C_CREATE_ROOM, C_JOIN_ROOM, C_SEARCH_ROOM, S_ROOM};
use tools::protos::server_protocol::{
    PlayerBattlePt, B_R_G_PUNISH_MATCH, B_S_SUMMARY, G_R_CREATE_ROOM, G_R_JOIN_ROOM,
    G_R_SEARCH_ROOM, R_G_SYNC_RANK,
};
use tools::util::packet::Packet;

///玩家基本数据结构体，用于封装例如玩家ID，昵称，创建时间等等
/// user_id:玩家ID
/// data：作为玩家具体数据，由jsonvalue封装
/// version：数据版本号，大于0则代表有改动，需要update到db
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct User {
    pub user_id: u32,      //玩家id
    pub ol: bool,          //是否在线
    pub nick_name: String, //玩家昵称
    pub grade: u32,        //玩家等级
    pub soul: u32,         //灵魂头像
    pub grade_frame: u32,  //grade像框
    #[serde(skip_serializing_if = "String::is_empty")]
    pub last_login_time: String, //上次登陆时间
    #[serde(skip_serializing_if = "String::is_empty")]
    pub last_off_time: String, //上次离线时间
    pub last_character: u32, //上次使用对角色
    pub total_online_time: u64, //总在线时间
    pub punish_match: PunishMatch, //匹配惩数据
    pub sync_time: u32,    //同步时间
    pub dlc: Vec<u32>,     //dlc
    #[serde(skip_serializing)]
    pub version: Cell<u32>, //数据版本号
}

///匹配惩罚数据
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PunishMatch {
    pub start_time: i64, //开始惩罚时间
    pub punish_id: u8,   //惩罚id
}

impl Into<PunishMatchPt> for PunishMatch {
    fn into(self) -> PunishMatchPt {
        let mut pmp = PunishMatchPt::new();
        pmp.punish_id = self.punish_id as u32;
        pmp.start_time = self.start_time;
        pmp
    }
}

impl From<&PunishMatchPt> for PunishMatch {
    fn from(pmp: &PunishMatchPt) -> Self {
        let mut pm = PunishMatch::default();
        pm.punish_id = pmp.punish_id as u8;
        pm.start_time = pmp.start_time;
        pm
    }
}

///为User实现Entiry
impl Entity for User {
    ///设置玩家id
    fn set_user_id(&mut self, user_id: u32) {
        self.user_id = user_id;
        self.add_version();
    }

    ///设置玩家id
    fn set_ids(&mut self, user_id: u32, _: u32) {
        self.user_id = user_id;
        self.add_version();
    }

    fn update_login_time(&mut self) {
        // let map = self.get_mut_json_value();
        // let time = Local::now();
        // let jv = JsonValue::String(time.naive_local().format("%Y-%m-%dT%H:%M:%S").to_string());
        // map.unwrap().insert("last_login_time".to_owned(), jv);
        // self.add_version();
    }

    fn update_off_time(&mut self) {
        let time = Local::now();
        let res = time.naive_local().format("%Y-%m-%dT%H:%M:%S").to_string();
        self.last_off_time = res;
        self.add_version();
    }

    fn day_reset(&mut self) {}
    fn add_version(&self) {
        let v = self.version.get() + 1;
        self.version.set(v);
    }
    fn clear_version(&self) {
        self.version.get();
    }
    fn get_version(&self) -> u32 {
        self.version.get()
    }

    fn get_tem_id(&self) -> Option<u32> {
        None
    }

    fn get_user_id(&self) -> u32 {
        self.user_id
    }

    fn get_data(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn init(data: serde_json::Value) -> Self
    where
        Self: Sized,
    {
        let user: User = serde_json::from_value(data).unwrap();
        user
    }
}

impl EntityData for User {
    fn try_clone_for_db(&self) -> Box<dyn EntityData> {
        let res = Box::new(self.clone());
        self.version.set(0);
        res
    }
}

impl Dao for User {
    //获得表名
    fn get_table_name(&self) -> &str {
        "t_u_player"
    }
}

impl User {
    pub fn set_grade(&mut self, grade: u32) {
        self.grade = grade;
        self.add_version();
    }

    #[allow(dead_code)]
    pub fn add_grade(&mut self) -> anyhow::Result<u32> {
        let res = self.grade;
        let mut grade = res as usize;
        grade += 1;
        let mut max_grade = 2_u32;
        let max_grade_temp = crate::TEMPLATES
            .get_constant_temp_mgr_ref()
            .temps
            .get("max_grade");
        match max_grade_temp {
            Some(max_grade_temp) => {
                let res = u32::from_str(max_grade_temp.value.as_str());
                match res {
                    Ok(res) => {
                        max_grade = res;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
            }
            None => {
                error!("max_grade is not find!");
            }
        }
        if grade as u32 > max_grade {
            grade = 1;
        }
        self.grade = grade as u32;
        self.add_version();
        Ok(grade as u32)
    }

    #[allow(dead_code)]
    pub fn sub_grade(&mut self) -> anyhow::Result<u32> {
        let res = self.grade;

        let mut grade = res as isize;
        grade -= 1;
        if grade <= 0 {
            grade = 1;
        }
        self.grade = grade as u32;
        self.add_version();
        Ok(grade as u32)
    }

    ///增加在线时间
    pub fn add_online_time(&mut self) {
        // let login_time = chrono::NaiveDateTime::from_str(self.last_login_time.as_str());
        // if let Err(_) = login_time {
        //     return;
        // }
        // let login_time = login_time.unwrap();
        // let res = login_time.timestamp() - Local::now().timestamp();
        // let res = (res / 1000) as usize;
        // let total_time = self.total_online_time;
        // let tmp = total_time + res as u64;
        // self.total_online_time = tmp;
        // self.add_version();
    }

    pub fn update_login(&mut self) {
        self.update_login_time();
        self.ol = true;
        self.add_version();
    }

    pub fn update_off(&mut self) {
        self.update_off_time();
        self.ol = false;
        self.add_online_time();
        self.add_version();
    }

    pub fn set_last_character(&mut self, cter_id: u32) {
        self.last_character = cter_id;
        self.add_version();
    }

    pub fn set_nick_name(&mut self, name: &str) {
        let str_key = "nick_name".to_owned();
        self.nick_name = name.to_owned();
        self.add_version();
        //修改redis
        modify_redis_user(self.user_id, str_key, JsonValue::from(name));
    }

    pub fn get_nick_name(&self) -> &str {
        self.nick_name.as_str()
    }

    pub fn set_dlc(&mut self, v: Vec<u32>) {
        if v.is_empty() {
            return;
        }
        self.dlc = v;
        self.add_version();
    }

    pub fn new(user_id: u32, nick_name: &str) -> Self {
        let mut user = User::default();
        user.user_id = user_id;
        user.grade = 1;
        user.nick_name = nick_name.to_owned();
        user
    }

    pub fn query(table_name: &str, user_id: u32, tem_id: Option<u32>) -> Option<Self> {
        let mut v: Vec<Value> = Vec::new();
        v.push(Value::UInt(user_id as u64));

        let mut sql = String::new();
        sql.push_str("select * from ");
        sql.push_str(table_name);
        sql.push_str(" where user_id=:user_id");
        if tem_id.is_some() {
            sql.push_str(" and tem_id:tem_id");
        }

        let q: Result<QueryResult, Error> = DB_POOL.exe_sql(sql.as_str(), Some(v));
        if q.is_err() {
            error!("{:?}", q.err().unwrap());
            return None;
        }
        let q = q.unwrap();

        let mut data = None;
        for _qr in q {
            let (_, js): (u32, serde_json::Value) = mysql::from_row(_qr.unwrap());
            let u = User::init(js);
            data = Some(u);
        }
        data
    }
}

///请求修改昵称
#[warn(unreachable_code)]
pub fn modify_nick_name(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let user = gm.users.get_mut(&user_id);
    if user.is_none() {
        let str = format!("user data is null for id:{}", user_id);
        error!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    let mut s_s_d = S_MODIFY_NICK_NAME::new();
    let mut cmn = C_MODIFY_NICK_NAME::new();
    let res = cmn.merge_from_bytes(packet.get_data());

    let mut is_success = true;
    if res.is_err() {
        is_success = false;
        error!(
            "protobuf:C_MODIFY_NICK_NAME parse has error!cmd:{}",
            packet.get_cmd()
        );
        s_s_d.err_mess = "request data is error!".to_owned();
    }
    let user = user.unwrap();

    if cmn.nick_name.as_str() == user.get_user_info_mut_ref().get_nick_name() {
        is_success = false;
        error!("nick_name has no change!cmd:{}", packet.get_cmd());
        s_s_d.err_mess = "nick_name has no change!".to_owned();
    }
    if is_success {
        user.get_user_info_mut_ref()
            .set_nick_name(cmn.nick_name.as_str());
    }
    s_s_d.is_succ = is_success;

    gm.send_2_client(
        ClientCode::NickNameModify,
        user_id,
        s_s_d.write_to_bytes().unwrap(),
    );
    info!("执行修改昵称函数!");
    Ok(())
}

///创建房间
pub fn create_room(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    let user_data = gm.users.get(&user_id);

    let mut s_r = S_ROOM::new();
    if user_data.is_none() {
        let str = format!("this player is not login!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        s_r.is_succ = false;
        s_r.err_mess = str.clone();
        gm.send_2_client(ClientCode::Room, user_id, s_r.write_to_bytes().unwrap());
        return Ok(());
    }

    //解析客户端发过来的参数
    let mut cr = C_CREATE_ROOM::new();
    cr.merge_from_bytes(packet.get_data())?;
    let room_type = cr.get_room_type();
    //封装proto发送给房间服
    let mut gr = G_R_CREATE_ROOM::new();
    gr.set_room_type(room_type);
    let mut pbp = PlayerBattlePt::new();
    let user_data = user_data.unwrap();
    let user_info = user_data.get_user_info_ref();
    pbp.set_user_id(user_id);
    pbp.set_nick_name(user_info.get_nick_name().to_owned());
    pbp.set_grade(user_info.grade);
    pbp.set_grade_frame(user_info.grade_frame);
    pbp.set_soul(user_info.soul);
    //封装玩家排行积分
    let lp = user_data.get_league_ref().into_league_pt();
    pbp.set_league(lp);
    let punish_match_pt = user_info.punish_match.into();
    pbp.set_punish_match(punish_match_pt);
    //封装角色
    for cter in user_data.get_characters_ref().cter_map.values() {
        let cter_pt = cter.clone().into();
        pbp.cters.push(cter_pt);
    }
    gr.set_pbp(pbp);
    //发给房间
    gm.send_2_server(
        RoomCode::CreateRoom.into_u32(),
        user_id,
        gr.write_to_bytes().unwrap(),
    );
    Ok(())
}

///创建房间
pub fn join_room(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    let user_data = gm.users.get(&user_id);

    let mut s_r = S_ROOM::new();
    if user_data.is_none() {
        let str = format!("this player is not login!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        s_r.is_succ = false;
        s_r.err_mess = str.clone();
        gm.send_2_client(ClientCode::Room, user_id, s_r.write_to_bytes()?);
        anyhow::bail!(str)
    }

    let mut cjr = C_JOIN_ROOM::new();
    let res = cjr.merge_from_bytes(packet.get_data());
    if res.is_err() {
        error!("{:?}", res.err().unwrap());
        return Ok(());
    }

    let user_data = user_data.unwrap();
    let user_info = user_data.get_user_info_ref();
    let mut pbp = PlayerBattlePt::new();
    pbp.set_user_id(user_id);
    pbp.set_nick_name(user_info.get_nick_name().to_owned());
    pbp.set_grade(user_info.grade);
    pbp.set_grade_frame(user_info.grade_frame);
    pbp.set_soul(user_info.soul);
    //封装玩家排行积分
    let lp = user_data.get_league_ref().into_league_pt();
    pbp.set_league(lp);
    let punish_match_pt = user_info.punish_match.into();
    pbp.set_punish_match(punish_match_pt);
    for cter in user_data.get_characters_ref().cter_map.values() {
        pbp.cters.push(cter.clone().into());
    }
    let mut grj = G_R_JOIN_ROOM::new();
    grj.set_room_id(cjr.room_id);
    grj.set_pbp(pbp);
    //发给房间
    gm.send_2_server(
        RoomCode::JoinRoom.into_u32(),
        user_id,
        grj.write_to_bytes()?,
    );
    Ok(())
}

///修改grade相框和soul头像
pub fn modify_grade_frame_and_soul(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let user_data = gm.users.get_mut(&user_id);
    if user_data.is_none() {
        warn!("could not find user_data for user_id {}", user_id);
        return Ok(());
    }
    let user_data = user_data.unwrap();
    let mut proto = C_MODIFY_GRADE_FRAME_AND_SOUL::new();
    let res = proto.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let grade_frame = proto.get_grade_frame();
    let soul = proto.soul;
    let mut res_proto = S_MODIFY_GRADE_FRAME_AND_SOUL::new();
    res_proto.is_succ = true;
    if grade_frame == 0 && soul == 0 {
        res_proto.is_succ = false;
        res_proto.err_mess = "params is invalid!".to_owned();
        warn!("params is invalid!");
    }

    if grade_frame > 0 && !user_data.grade_frame.grade_frames.contains(&grade_frame) {
        let str = format!("this player do not have this grade_frame:{}!", grade_frame);
        res_proto.is_succ = false;
        res_proto.err_mess = str.clone();
        warn!("{:?}", str.as_str());
    } else {
        let res = user_data.get_user_info_mut_ref();
        res.grade_frame = grade_frame;
    }

    if soul > 0 && !user_data.soul.souls.contains(&soul) {
        let str = format!("this player do not have this soul:{}!", soul);
        res_proto.is_succ = false;
        res_proto.err_mess = str.clone();
        warn!("{:?}", str.as_str());
    } else {
        let res = user_data.get_user_info_mut_ref();
        res.soul = soul;
    }
    //返回客户端
    let bytes = res_proto.write_to_bytes();
    match bytes {
        Ok(bytes) => gm.send_2_client(ClientCode::ModifyGradeFrameAndSoul, user_id, bytes),
        Err(e) => {
            error!("{:?}", e)
        }
    }
    return Ok(());
}

///同步排行榜快照
pub fn show_rank(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    if gm.rank.is_empty() {
        warn!("the rank is empty!");
        return Ok(());
    }

    let user_data = gm.users.get_mut(&user_id);
    if user_data.is_none() {
        warn!("could not find user_data for user_id {}", user_id);
        return Ok(());
    }

    let mut ssr = S_SHOW_RANK::new();
    //封装自己的
    if !gm.user_rank.contains_key(&user_id){
        let res = gm.rank.par_iter().find_first(|x| x.user_id == user_id);
        if let Some(res) = res {
            let rip = RankInfoPtPtr(res as *const RankInfoPt);
            gm.user_rank.insert(user_id, rip);
        }
    }
    let res = gm.user_rank.get(&user_id);
    if let Some(res) = res {
        unsafe{
            let res = res.0.as_ref();
            if let Some(res) = res{
                ssr.set_self_rank(res.clone());
            }
        }
    }

    //封装另外100个
    for rank in gm.rank[0..100].iter() {
        ssr.ranks.push(rank.clone());
    }
    let bytes = ssr.write_to_bytes();
    if let Err(e) = bytes {
        error!("{:?}", e);
        return Ok(());
    }
    let bytes = bytes.unwrap();
    gm.send_2_client(ClientCode::ShowRank, user_id, bytes);
    Ok(())
}

///同步排行榜快照
pub fn sync_rank(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let mut rgsr = R_G_SYNC_RANK::new();
    let res = rgsr.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let rank = rgsr.ranks.to_vec();
    gm.rank = rank;
    gm.user_rank.clear();
    Ok(())
}

///更新玩家匹配惩罚数据
pub fn punish_match(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    let mut brg = B_R_G_PUNISH_MATCH::new();
    let res = brg.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let user_data = gm.users.get_mut(&user_id);
    if let None = user_data {
        warn!("could not find UserData for user_id {}", user_id);
        return Ok(());
    }
    let user_data = user_data.unwrap();
    let user = user_data.get_user_info_mut_ref();
    user.punish_match = PunishMatch::from(brg.get_punish_match());
    user.add_version();
    user_data.add_version();
    Ok(())
}

///匹配房间
pub fn search_room(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    let user_data = gm.users.get(&user_id);

    let mut s_r = S_ROOM::new();
    if user_data.is_none() {
        let str = format!("this player is not login!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        s_r.is_succ = false;
        s_r.err_mess = str.clone();
        gm.send_2_client(ClientCode::Room, user_id, s_r.write_to_bytes()?);
        anyhow::bail!(str)
    }

    let mut csr = C_SEARCH_ROOM::new();
    let res = csr.merge_from_bytes(packet.get_data());
    if res.is_err() {
        error!("{:?}", res.err().unwrap());
        return Ok(());
    }

    let user_data = user_data.unwrap();
    let user_info = user_data.get_user_info_ref();
    let mut pbp = PlayerBattlePt::new();
    pbp.set_user_id(user_id);
    pbp.set_nick_name(user_info.get_nick_name().to_owned());
    pbp.set_grade(user_info.grade);
    pbp.set_grade_frame(user_info.grade_frame);
    pbp.set_soul(user_info.soul);
    //封装玩家排行积分
    let lp = user_data.get_league_ref().into_league_pt();
    pbp.set_league(lp);
    for cter in user_data.get_characters_ref().cter_map.values() {
        pbp.cters.push(cter.clone().into());
    }
    let mut grs = G_R_SEARCH_ROOM::new();
    grs.set_room_type(csr.get_room_type());
    grs.set_pbp(pbp);
    //发给房间
    gm.send_2_server(
        RoomCode::SearchRoom.into_u32(),
        user_id,
        grs.write_to_bytes()?,
    );
    Ok(())
}

///房间战斗结算
pub fn summary(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let mut bgs = B_S_SUMMARY::new();
    let res = bgs.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let room_type = RoomType::try_from(bgs.room_type as u8);
    if let Err(e) = room_type {
        error!("{:?}", e);
        return Ok(());
    }
    let room_type = room_type.unwrap();
    let summary_data = bgs.get_summary_data();
    let user_id = summary_data.user_id;
    let cter_id = summary_data.cter_id;
    let grade = summary_data.get_grade();
    let res = gm.users.get_mut(&user_id);
    if let None = res {
        error! {"summary!UserData is not find! user_id:{}",user_id};
        return Ok(());
    }
    let user_data = res.unwrap();
    //处理统计
    let cters = user_data.get_characters_mut_ref().add_use_times(cter_id);
    user_data.league.set_cters(cters.clone());
    //处理持久化到数据库
    user_data.add_version();
    //如果是匹配房
    if room_type == RoomType::Match {
        //更新段位积分
        user_data.league.update_from_pt(summary_data.get_league());
        //第一名就加grade
        user_data.user_info.set_grade(grade);
        bgs.cters.extend_from_slice(cters.as_slice());
        if user_data.league.id > 0{
            //更新排行榜
            gm.send_2_server(RankCode::UpdateRank.into_u32(), user_id, bgs.write_to_bytes()?);
        }
    }
    Ok(())
}
