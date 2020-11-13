use super::*;
use crate::helper::redis_helper::modify_redis_user;
use chrono::Local;
use protobuf::Message;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use tools::cmd_code::{ClientCode, RoomCode};
use tools::protos::protocol::{C_MODIFY_NICK_NAME, S_MODIFY_NICK_NAME};
use tools::protos::room::{C_CREATE_ROOM, C_JOIN_ROOM, C_SEARCH_ROOM, S_ROOM};
use tools::protos::server_protocol::{
    PlayerBattlePt, G_R_CREATE_ROOM, G_R_JOIN_ROOM, G_R_SEARCH_ROOM, R_G_SUMMARY,
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
    #[serde(skip_serializing_if = "String::is_empty")]
    pub last_login_time: String, //上次登陆时间
    #[serde(skip_serializing_if = "String::is_empty")]
    pub last_off_time: String, //上次离线时间
    pub last_character: u32, //上次使用对角色
    pub total_online_time: u64, //总在线时间
    pub sync_time: u32,    //同步时间
    pub dlc: Vec<u32>,     //dlc
    #[serde(skip_serializing)]
    pub version: Cell<u32>, //数据版本号
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
    fn try_clone(&self) -> Box<dyn EntityData> {
        Box::new(self.clone())
    }
}

impl Dao for User {
    //获得表名
    fn get_table_name(&self) -> &str {
        "t_u_player"
    }
}

impl User {
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
    let mut gr = G_R_CREATE_ROOM::new();
    gr.set_room_type(cr.room_type);
    let mut pbp = PlayerBattlePt::new();
    let user_data = user_data.unwrap();
    pbp.set_user_id(user_id);
    pbp.set_nick_name(user_data.get_user_info_ref().get_nick_name().to_owned());
    for cter in user_data.get_characters_ref().cter_map.values() {
        let cter_pt = cter.clone().into();
        pbp.cters.push(cter_pt);
    }
    gr.set_pbp(pbp);
    //发给房间
    gm.send_2_room(RoomCode::CreateRoom, user_id, gr.write_to_bytes().unwrap());
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
    let mut pbp = PlayerBattlePt::new();
    pbp.set_user_id(user_id);
    pbp.set_nick_name(user_data.get_user_info_ref().get_nick_name().to_owned());
    for cter in user_data.get_characters_ref().cter_map.values() {
        pbp.cters.push(cter.clone().into());
    }
    let mut grj = G_R_JOIN_ROOM::new();
    grj.set_room_id(cjr.room_id);
    grj.set_pbp(pbp);
    //发给房间
    gm.send_2_room(RoomCode::JoinRoom, user_id, grj.write_to_bytes()?);
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
    let mut pbp = PlayerBattlePt::new();
    pbp.set_user_id(user_id);
    pbp.set_nick_name(user_data.get_user_info_ref().get_nick_name().to_owned());
    for cter in user_data.get_characters_ref().cter_map.values() {
        pbp.cters.push(cter.clone().into());
    }
    let mut grs = G_R_SEARCH_ROOM::new();
    grs.set_room_type(csr.get_room_type());
    grs.set_pbp(pbp);
    //发给房间
    gm.send_2_room(RoomCode::SearchRoom, user_id, grs.write_to_bytes()?);
    Ok(())
}

///房间战斗结算
pub fn summary(gm: &mut GameMgr, packet: Packet) -> anyhow::Result<()> {
    let mut rgs = R_G_SUMMARY::new();
    let res = rgs.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }

    for summary_data in rgs.get_summary_datas() {
        let res = gm.users.get_mut(&summary_data.user_id);
        if let None = res {
            error! {"summary!UserData is not find! user_id:{}",summary_data.user_id};
            continue;
        }
        let user_data = res.unwrap();
        let res = user_data
            .get_characters_mut_ref()
            .cter_map
            .get_mut(&summary_data.cter_id);
        if let None = res {
            error! {"summary!Character is not find! user_id:{},cter_id:{}",summary_data.user_id,summary_data.cter_id};
            continue;
        }
        let cter = res.unwrap();
        let res;
        //第一名就加grade
        if summary_data.rank == 0 {
            res = cter.add_grade();
        } else {
            //否则就减grade
            res = cter.sub_grade();
        }
        if let Err(e) = res {
            error!("{:?}", e);
        }
    }
    Ok(())
}
