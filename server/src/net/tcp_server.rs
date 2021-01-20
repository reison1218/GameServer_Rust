use tools::protos::base::{PlayerPt, PunishMatchPt, ResourcesPt};
use tools::tcp::TcpSender;

use crate::entity::character::Characters;
use crate::entity::league::League;
use crate::entity::user::{insert_characters, insert_league, insert_user, UserData};
use crate::entity::user_info::User;
use crate::helper::redis_helper::get_user_from_redis;
use crate::mgr::game_mgr::GameMgr;
use async_std::sync::Mutex;
use async_std::task::block_on;
use async_trait::async_trait;
use log::{error, info, warn};
use protobuf::Message;
use std::str::FromStr;
use std::sync::Arc;
use tools::cmd_code::{ClientCode, GameCode, ServerCommonCode};
use tools::protos::protocol::{C_USER_LOGIN, S_USER_LOGIN};
use tools::util::packet::Packet;

#[derive(Clone)]
struct TcpServerHandler {
    gm: Arc<Mutex<GameMgr>>,
}

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

#[async_trait]
impl tools::tcp::Handler for TcpServerHandler {
    async fn try_clone(&self) -> Self {
        self.clone()
    }

    async fn on_open(&mut self, sender: TcpSender) {
        let mut lock = self.gm.lock().await;
        lock.set_sender(sender);
        lock.init_rank();
    }

    async fn on_close(&mut self) {
        info!("与tcp客户端断开连接");
    }

    async fn on_message(&mut self, mess: Vec<u8>) {
        let packet_array = Packet::build_array_from_server(mess);

        if let Err(e) = packet_array {
            error!("{:?}", e);
            return;
        }
        let packet_array = packet_array.unwrap();

        for packet in packet_array {
            let gm = self.gm.clone();
            async_std::task::spawn(handler_mess_s(gm, packet));
        }
    }
}

async fn handler_mess_s(gm: Arc<Mutex<GameMgr>>, packet: Packet) {
    //如果为空，什么都不执行
    if packet.get_cmd() != GameCode::Login.into_u32()
        && packet.get_cmd() != GameCode::UnloadUser.into_u32()
        && packet.get_cmd() != ServerCommonCode::ReloadTemps.into_u32()
        && packet.get_data().is_empty()
    {
        error!("packet bytes is null!cmd:{}", packet.get_cmd());
        return;
    }
    //判断是否执行登录
    if packet.get_cmd() == GameCode::Login.into_u32() {
        let mut c_login = C_USER_LOGIN::new();
        let result = c_login.merge_from_bytes(packet.get_data());

        if let Err(e) = result {
            error!("{:?}", e);
            return;
        }
        //执行登录
        let result = login(gm, packet).await;
        if let Err(e) = result {
            error!("{:?}", e);
            return;
        }
    } else {
        //不登录就执行其他命令
        let res = gm.lock().await.invok(packet);
        match res {
            Ok(_) => {}
            Err(e) => {
                error!("{:?}", e.to_string());
            }
        }
    }
}

//登录函数，执行登录
async fn login(gm: Arc<Mutex<GameMgr>>, packet: Packet) -> anyhow::Result<()> {
    //玩家id
    let user_id = packet.get_user_id();
    let mut gm_lock = gm.lock().await;
    let user_data = gm_lock.users.contains_key(&user_id);
    //走登录流程
    //如果内存没有数据，则从数据库里面找
    if !user_data {
        //初始化玩家数据
        let user_data = init_user_data(user_id)?;
        gm_lock.users.insert(user_id, user_data);
    }
    //封装会话
    let user_data = gm_lock.users.get_mut(&user_id);
    if user_data.is_none() {
        let str = format!("there is no data for userid:{}", &user_id);
        anyhow::bail!(str)
    }
    let user_data = user_data.unwrap();

    let user = user_data.get_user_info_mut_ref();
    user.update_login();

    //返回客户端
    let lr = user2proto(user_data);
    gm_lock.send_2_client(ClientCode::Login, user_id, lr.write_to_bytes()?);
    info!("用户完成登录！user_id:{}", &user_id);
    Ok(())
}

///初始化玩家数据
fn init_user_data(user_id: u32) -> anyhow::Result<UserData> {
    //判断redis里面有没有,用户中心没有则直接代表不合法，不与执行
    let value = get_user_from_redis(user_id);
    if value.is_none() {
        let str = format!("redis has no data for user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }

    let mut ud = UserData::init_from_db(user_id);

    //数据库没有则创建新号
    if ud.is_none() {
        let json_value = value.unwrap();
        let nick_name = json_value.get("nick_name");
        if nick_name.is_none() {
            let str = format!("nick_name is none for user_id:{}", user_id);
            error!("{:?}", str.as_str());
            anyhow::bail!(str)
        }
        let mut user = User::new(user_id, nick_name.unwrap().as_str().unwrap());
        //以下入库采用异步执行，以免造成io堵塞
        //玩家角色数据
        let c = Characters::new(user.user_id);
        //玩家段位数据
        let league = League::new(user.user_id, user.nick_name.clone());
        user.set_last_character(c.get_frist());

        //封装到userdata里
        ud = Some(UserData::new(user.clone(), c.clone(), league.clone()));

        //异步持久化到db
        async_std::task::spawn(insert_user(user));
        async_std::task::spawn(insert_characters(c));
        async_std::task::spawn(insert_league(league));
    }
    Ok(ud.unwrap())
}

///user结构体转proto
fn user2proto(user: &mut UserData) -> S_USER_LOGIN {
    let mut lr = S_USER_LOGIN::new();
    lr.set_is_succ(true);
    // let result = user.get_json_value("signInTime");
    // if result.is_some() {
    //     let str = result.unwrap().as_str().unwrap();
    //     let mut sign_in_Time = str.parse::<NaiveDateTime>();
    //     lr.signInTime = sign_in_Time.unwrap().timestamp_subsec_micros();
    // }

    let mut time = user.get_user_info_mut_ref().sync_time;
    lr.sync_time = time;
    let mut ppt = PlayerPt::new();
    let nick_name = user.get_user_info_mut_ref().nick_name.as_str();
    ppt.set_nick_name(nick_name.to_string());
    let last_character = user.get_user_info_ref().last_character;
    ppt.set_last_character(last_character);
    ppt.dlc.push(1);
    let mut punish_match_pt: PunishMatchPt = user.get_user_info_mut_ref().punish_match.into();
    punish_match_pt.start_time /= 1000;
    ppt.set_punish_match(punish_match_pt);
    lr.player_pt = protobuf::SingularPtrField::some(ppt);
    time = 0;
    let res =
        chrono::NaiveDateTime::from_str(user.get_user_info_mut_ref().last_login_time.as_str());
    if let Ok(res) = res {
        time = res.timestamp_subsec_micros();
    }
    lr.last_login_time = time;
    time = 0;
    let res = chrono::NaiveDateTime::from_str(user.get_user_info_mut_ref().last_off_time.as_str());
    if let Ok(res) = res {
        time = res.timestamp_subsec_micros();
    }

    lr.last_logoff_time = time;

    let mut v = Vec::new();
    let mut res = ResourcesPt::new();
    res.id = 1;
    res.field_type = 1;
    res.num = 100_u32;
    v.push(res);

    let resp = protobuf::RepeatedField::from(v);
    lr.resp = resp;

    let mut c_v = Vec::new();
    for cter in user.get_characters_ref().cter_map.values() {
        c_v.push(cter.clone().into());
    }
    let res = protobuf::RepeatedField::from(c_v);
    lr.set_cters(res);
    lr
}

pub fn new(address: &str, gm: Arc<Mutex<GameMgr>>) {
    let sh = TcpServerHandler { gm };
    let res = tools::tcp::tcp_server::new(address.to_string(), sh);
    let res = block_on(res);
    if let Err(e) = res {
        error!("{:?}", e);
        std::process::abort();
    }
}
