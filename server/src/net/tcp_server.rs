use tools::protos::base::{PlayerPt, ResourcesPt};
use tools::tcp::TcpSender;

use crate::entity::character::Characters;
use crate::entity::user::{insert_characters, insert_user, UserData};
use crate::entity::user_contants::*;
use crate::entity::user_info::User;
use crate::entity::Entity;
use crate::helper::redis_helper::get_user_from_redis;
use crate::mgr::game_mgr::GameMgr;
use log::{error, info, warn};
use protobuf::Message;
use std::sync::{Arc, Mutex};
use tools::cmd_code::{ClientCode, GameCode};
use tools::protos::protocol::{C_USER_LOGIN, S_USER_LOGIN};
use tools::util::packet::Packet;

#[derive(Clone)]
struct TcpServerHandler {
    sender: Option<TcpSender>,
    gm: Arc<Mutex<GameMgr>>,
}

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

impl tools::tcp::Handler for TcpServerHandler {
    fn try_clone(&self) -> Self {
        let mut sender: Option<TcpSender> = None;
        if self.sender.is_some() {
            sender = Some(self.sender.as_ref().unwrap().clone());
        }
        TcpServerHandler {
            sender: sender,
            gm: self.gm.clone(),
        }
    }

    fn on_open(&mut self, sender: TcpSender) {
        self.gm.lock().unwrap().set_sender(sender);
    }

    fn on_close(&mut self) {
        info!("与tcp客户端断开连接");
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        let packet_array = Packet::build_array_from_server(mess);

        if let Err(e) = packet_array {
            error!("{:?}", e);
            return;
        }
        let packet_array = packet_array.unwrap();

        for packet in packet_array {
            //判断是否是房间服的命令，如果不是，则直接无视掉
            if packet.get_cmd() < GameCode::Min as u32 || packet.get_cmd() > GameCode::Max as u32 {
                error!("the cmd:{} is not belong gameserver!", packet.get_cmd());
                continue;
            }
            let gm = self.gm.clone();
            async_std::task::spawn(handler_mess_s(gm, packet));
        }
    }
}

async fn handler_mess_s(gm: Arc<Mutex<GameMgr>>, packet: Packet) {
    //如果为空，什么都不执行
    if packet.get_cmd() != GameCode::Login as u32
        && packet.get_cmd() != GameCode::LineOff as u32
        && packet.get_data().is_empty()
    {
        error!("packet bytes is null!");
        return;
    }
    //判断是否执行登录
    if packet.get_cmd() == GameCode::Login as u32 {
        let mut c_login = C_USER_LOGIN::new();
        let result = c_login.merge_from_bytes(packet.get_data());

        if let Err(e) = result {
            error!("{:?}", e);
            return;
        }
        //执行登录
        let result = login(gm, packet);
        if let Err(e) = result {
            error!("{:?}", e);
            return;
        }
    } else {
        //不登录就执行其他命令
        let res = gm.lock().unwrap().invok(packet);
        match res {
            Ok(_) => {}
            Err(e) => {
                error!("{:?}", e.to_string());
            }
        }
    }
}

//登录函数，执行登录
fn login(gm: Arc<Mutex<GameMgr>>, packet: Packet) -> anyhow::Result<()> {
    //玩家id
    let user_id = packet.get_user_id();
    let user_data = gm.lock().unwrap().users.contains_key(&user_id);
    //走登录流程
    let mut gm_lock = gm.lock().unwrap();
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
        user.set_last_character(c.get_frist());

        //封装到userdata里
        ud = Some(UserData::new(user.clone(), c.clone()));

        //异步持久化到db
        async_std::task::spawn(insert_user(user));
        async_std::task::spawn(insert_characters(c));
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

    let mut result = user.get_user_info_mut_ref().get_time(SYNC_TIME);
    let mut time = 0_u32;
    if result.is_some() {
        time = result.unwrap().timestamp_subsec_micros();
    }
    lr.sync_time = time;
    let mut ppt = PlayerPt::new();
    let nick_name = user
        .get_user_info_mut_ref()
        .get_json_value(NICK_NAME)
        .unwrap()
        .as_str()
        .unwrap();
    ppt.set_nick_name(nick_name.to_string());
    let last_character = user.get_user_info_ref().get_usize(LAST_CHARACTER);
    if last_character.is_none() {
        ppt.set_last_character(0);
    } else {
        ppt.set_last_character(last_character.unwrap() as u32);
    }
    ppt.dlc.push(1);
    lr.player_pt = protobuf::SingularPtrField::some(ppt);
    result = user.get_user_info_mut_ref().get_time(LAST_LOGIN_TIME);
    time = 0;
    if result.is_some() {
        time = result.unwrap().timestamp_subsec_micros();
    }

    lr.last_login_time = time;

    time = 0;
    result = user.get_user_info_mut_ref().get_time(OFF_TIME);
    if result.is_some() {
        time = result.unwrap().timestamp_subsec_micros();
    }

    lr.last_logoff_time = time;

    let mut v = Vec::new();
    let mut res = ResourcesPt::new();
    res.id = 1;
    res.field_type = 1;
    res.num = 100 as u32;
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
    let sh = TcpServerHandler {
        sender: None,
        gm: gm,
    };
    tools::tcp::tcp_server::new(address, sh).unwrap();
}
