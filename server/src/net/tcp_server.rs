use super::*;
use chrono::{Duration, Local, NaiveDateTime, NaiveTime};
use std::io::Write;
use tools::thread_pool::ThreadPoolHandler;

use tools::protos::base::ResourcesPt;
use tools::tcp::TcpSender;

use crate::db::table_contants;
use crate::entity::user::UserData;
use crate::entity::Entity;
use crate::helper::redis_helper::get_user_from_redis;
use crate::net::http::notice_user_center;
use crate::DB_POOL;
use futures::executor::block_on;
use protobuf::Message;
use std::str::FromStr;
use std::sync::mpsc::{Sender, SyncSender};
use tools::cmd_code::{ClientCode, GameCode};
use tools::protos::protocol::C_USER_LOGIN;

#[derive(Clone)]
struct TcpServerHandler {
    sender: Option<TcpSender>,
    gm: Arc<RwLock<GameMgr>>,
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
        self.gm.write().unwrap().sender = Some(sender);
    }

    fn on_close(&mut self) {
        info!("与tcp客户端断开连接");
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        let packet = Packet::from_only_server(mess);
        match packet {
            Ok(p) => {
                let mut gm = self.gm.clone();
                async_std::task::spawn(handler_mess_s(gm, p));
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
}

async fn handler_mess_s(gm: Arc<RwLock<GameMgr>>, packet: Packet) {
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
        let mut c_login = C_USER_LOGIN_PROTO::new();
        let result = c_login.merge_from_bytes(packet.get_data());
        if result.is_err() {
            error!("{:?}", result.err().unwrap());
            return;
        }
        //执行登录
        login(gm, packet);
    } else {
        //不登录就执行其他命令
        gm.write().unwrap().invok(packet);
    }
}

//登录函数，执行登录
fn login(gm: Arc<RwLock<GameMgr>>, mut packet: Packet) {
    //玩家id
    let user_id = packet.get_user_id();
    let mut user_data = false;
    {
        user_data = gm.read().unwrap().users.contains_key(&user_id);
    }
    //走登录流程
    let mut gm_lock = gm.write().unwrap();
    //如果内存没有数据，则从数据库里面找
    if !user_data {
        //判断redis里面有没有,用户中心没有则直接代表不合法，不与执行
        let value = get_user_from_redis(user_id);
        if value.is_none() {
            warn!("redis has no data for user_id:{}", user_id);
            return;
        }

        let mut user = User::query(USER, user_id, None);
        //数据库没有则创建新号
        if user.is_none() {
            let json_value = value.unwrap();
            let nick_name = json_value.get("nick_name");
            let avatar = json_value.get("avatar");
            if nick_name.is_none() || avatar.is_none() {
                error!("nick_name or avatar is none for user_id:{}", user_id);
                return;
            }
            user = Some(User::new(
                user_id,
                avatar.unwrap().as_str().unwrap(),
                nick_name.unwrap().as_str().unwrap(),
            ));
            //以下入库采用异步执行，以免造成io堵塞
            let mut user_mut = user.as_mut().unwrap().clone();
            async_std::task::spawn(insert_user(user_mut));
        }
        //封装到内存中
        gm_lock.users.insert(user_id, UserData::new(user.unwrap()));
    }
    //封装会话
    let user = gm_lock.users.get_mut(&user_id);
    if user.is_none() {
        error!("there is no data for userid:{}", &user_id);
        return;
    }

    let user = user.unwrap().get_user_info_mut_ref();
    user.update_login_time();

    //通知用户中心
    async_std::task::spawn(notice_user_center(user_id, "login"));

    //返回客户端
    let mut lr = user2proto(user);
    let bytes = lr.write_to_bytes().unwrap();
    packet.set_user_id(user_id);
    packet.set_is_client(true);
    packet.set_cmd(ClientCode::Login as u32);
    packet.set_data_from_vec(bytes);

    let result = gm_lock
        .sender
        .as_mut()
        .unwrap()
        .write(packet.build_server_bytes());
    info!("用户完成登录！user_id:{}", &user_id);
}

async fn insert_user(mut user: User) {
    info!("玩家数据不存在,现在创建新玩家:{}", user.user_id);
    let result = User::insert(&mut user);
    if result.is_err() {
        error!("{:?}", result.err().unwrap());
    }
}

///user结构体转proto
fn user2proto(user: &mut User) -> S_USER_LOGIN_PROTO {
    let mut lr = S_USER_LOGIN_PROTO::new();
    lr.set_is_succ(true);
    // let result = user.get_json_value("signInTime");
    // if result.is_some() {
    //     let str = result.unwrap().as_str().unwrap();
    //     let mut sign_in_Time = str.parse::<NaiveDateTime>();
    //     lr.signInTime = sign_in_Time.unwrap().timestamp_subsec_micros();
    // }

    let mut result = user.get_time(SYNC_TIME);
    let mut time = 0 as u32;
    if result.is_some() {
        time = result.unwrap().timestamp_subsec_micros();
    }
    lr.sync_time = time;
    let mut ppt = PlayerPt::new();
    let mut nick_name = user.get_json_value(NICK_NAME).unwrap().as_str().unwrap();
    ppt.set_nick_name(nick_name.to_string());
    ppt.dlc.push(1);
    lr.player_pt = protobuf::SingularPtrField::some(ppt);
    result = user.get_time(LAST_LOGIN_TIME);
    time = 0;
    if result.is_some() {
        time = result.unwrap().timestamp_subsec_micros();
    }

    lr.last_login_time = time;

    time = 0;
    result = user.get_time(OFF_TIME);
    if result.is_some() {
        time = result.unwrap().timestamp_subsec_micros();
    }

    lr.last_logoff_time = time;

    let mut v = Vec::new();
    for i in 0..1000 {
        let mut res = ResourcesPt::new();
        res.id = 1;
        res.field_type = 1;
        res.num = 100 as u32;
        v.push(res);
    }

    let resp = protobuf::RepeatedField::from(v);
    lr.resp = resp;
    lr
}

pub fn new(address: &str, gm: Arc<RwLock<GameMgr>>) {
    let sh = TcpServerHandler {
        sender: None,
        gm: gm,
    };
    tools::tcp::tcp_server::new(address, sh).unwrap();
}
