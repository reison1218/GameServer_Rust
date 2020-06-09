use super::*;
use crate::entity::user_contants::*;
use crate::helper::redis_helper::modify_redis_user;
use chrono::Local;
use protobuf::Message;
use tools::cmd_code::{ClientCode, RoomCode};
use tools::protos::base::CharacterPt;
use tools::protos::protocol::{C_MODIFY_NICK_NAME, S_MODIFY_NICK_NAME};
use tools::protos::room::S_ROOM;
use tools::protos::server_protocol::PlayerBattlePt;
use tools::util::packet::Packet;

///玩家基本数据结构体，用于封装例如玩家ID，昵称，创建时间等等
/// user_id:玩家ID
/// data：作为玩家具体数据，由jsonvalue封装
/// version：数据版本号，大于0则代表有改动，需要update到db
#[derive(Debug, Clone, Default)]
pub struct User {
    pub user_id: u32,    //玩家id
    pub data: JsonValue, //数据
    pub version: u32,    //数据版本号
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
        let map = self.get_mut_json_value();
        let time = Local::now();
        let jv = JsonValue::String(time.naive_local().format("%Y-%m-%dT%H:%M:%S").to_string());
        map.unwrap().insert("last_login_time".to_owned(), jv);
        self.add_version();
    }

    fn day_reset(&mut self) {}
    fn add_version(&mut self) {
        self.version += 1;
    }
    fn clear_version(&mut self) {
        self.version = 0;
    }
    fn get_version(&self) -> u32 {
        self.version
    }

    fn get_tem_id(&self) -> Option<u32> {
        None
    }

    fn get_user_id(&self) -> u32 {
        self.user_id
    }

    fn get_data(&self) -> &JsonValue {
        &self.data
    }

    fn get_data_mut(&mut self) -> &mut JsonValue {
        &mut self.data
    }

    fn init(user_id: u32, _: Option<u32>, js: JsonValue) -> Self
    where
        Self: Sized,
    {
        let u = User {
            user_id: user_id,
            data: js,
            version: 0,
        };
        u
    }
}

impl EntityData for User {
    fn try_clone(&self) -> Box<dyn EntityData> {
        let user = User::init(self.get_user_id(), None, self.data.clone());
        Box::new(user)
    }
}

impl Dao for User {
    //获得表名
    fn get_table_name(&mut self) -> &str {
        "t_u_player"
    }
}

impl User {
    pub fn set_last_character(&mut self, cter_id: u32) {
        let map = self.get_mut_json_value().unwrap();
        map.insert(LAST_CHARACTER.to_owned(), serde_json::Value::from(cter_id));
        self.version += 1;
    }

    pub fn set_nick_name(&mut self, name: &str) {
        let map = self.get_mut_json_value().unwrap();
        map.insert(NICK_NAME.to_owned(), serde_json::Value::from(name));
        self.version += 1;
        //修改redis
        modify_redis_user(self.user_id, NICK_NAME.to_string(), JsonValue::from(name));
    }

    pub fn get_nick_name(&self) -> &str {
        let nick_name = self.get_json_value(NICK_NAME);
        if nick_name.is_none() {
            return "";
        }
        nick_name.unwrap().as_str().unwrap()
    }

    pub fn set_dlc(&mut self, v: Vec<u32>) {
        if v.is_empty() {
            return;
        }
        let map = self.get_mut_json_value().unwrap();
        map.insert(DLC.to_owned(), serde_json::Value::from(v));
        self.version += 1;
    }

    pub fn new(user_id: u32, nick_name: &str) -> Self {
        let mut js_data = serde_json::map::Map::new();
        js_data.insert(USER_OL.to_string(), JsonValue::from(1));
        js_data.insert(NICK_NAME.to_string(), JsonValue::from(nick_name));
        let user = User::init(user_id, None, serde_json::Value::from(js_data));
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
            let (id, js) = mysql::from_row(_qr.unwrap());
            let u = User::init(id, tem_id, js);
            data = Some(u);
        }
        data
    }
}

///请求修改昵称
#[warn(unreachable_code)]
pub fn modify_nick_name(gm: &mut GameMgr, mut packet: Packet) -> anyhow::Result<()> {
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
    packet.set_cmd(ClientCode::NickNameModify as u32);
    packet.set_is_client(true);
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
    packet.set_data_from_vec(s_s_d.write_to_bytes()?);
    gm.sender
        .as_mut()
        .unwrap()
        .write(packet.build_server_bytes())?;
    info!("执行修改昵称函数!");
    Ok(())
}

///创建房间
pub fn create_room(gm: &mut GameMgr, mut packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut pbp = PlayerBattlePt::new();
    let user_data = gm.users.get(&user_id);

    let mut s_r = S_ROOM::new();
    if user_data.is_none() {
        let str = format!("this player is not login!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        s_r.is_succ = false;
        s_r.err_mess = str.clone();
        packet.set_data_from_vec(s_r.write_to_bytes().unwrap());
        let res = gm
            .sender
            .as_mut()
            .unwrap()
            .write(packet.build_client_bytes());
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
        }
        Ok(())
    }

    let user_data = user_data.unwrap();
    pbp.set_user_id(user_id);
    pbp.set_nick_name(user_data.get_user_info_ref().get_nick_name().to_owned());
    for (cter_id, cter) in user_data.get_characters_ref().cter_map.iter() {
        let mut cter_pt = CharacterPt::new();
        cter_pt.set_temp_id(*cter_id);
        cter_pt.set_skills(cter.get_skills()?);
        pbp.cters.push(cter_pt);
    }
    //发给房间
    packet.set_cmd(RoomCode::CreateRoom as u32);
    packet.set_is_client(false);

    let res = gm
        .sender
        .as_mut()
        .unwrap()
        .write(packet.build_server_bytes());
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
    }
    Ok(())
}

///创建房间
pub fn join_room(gm: &mut GameMgr, mut packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut pbp = PlayerBattlePt::new();
    let user_data = gm.users.get(&user_id);

    let mut s_r = S_ROOM::new();
    if user_data.is_none() {
        let str = format!("this player is not login!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        s_r.is_succ = false;
        s_r.err_mess = str.clone();
        packet.set_cmd(ClientCode::Room as u32);
        packet.set_data_from_vec(s_r.write_to_bytes()?);
        gm.sender
            .as_mut()
            .unwrap()
            .write(packet.build_client_bytes())?;
        anyhow::bail!(str)
    }

    let user_data = user_data.unwrap();
    pbp.set_user_id(user_id);
    pbp.set_nick_name(user_data.get_user_info_ref().get_nick_name().to_owned());
    for (cter_id, cter) in user_data.get_characters_ref().cter_map.iter() {
        let mut cter_pt = CharacterPt::new();
        cter_pt.set_temp_id(*cter_id);
        cter_pt.set_skills(cter.get_skills()?);
        pbp.cters.push(cter_pt);
    }
    //发给房间
    packet.set_cmd(RoomCode::JoinRoom as u32);
    packet.set_is_client(false);

    gm.sender
        .as_mut()
        .unwrap()
        .write(packet.build_server_bytes())?;
    Ok(())
}
