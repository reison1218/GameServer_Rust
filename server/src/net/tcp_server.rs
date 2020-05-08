use super::*;
use chrono::{Duration, Local, NaiveDateTime, NaiveTime};
use std::io::Write;
use tools::thread_pool::ThreadPoolHandler;

use tools::protos::base::ResourcesPt;
use tools::tcp::TcpSender;

use crate::db::table_contants;
use crate::entity::user::UserData;
use crate::entity::Entity;
use crate::DB_POOL;
use futures::executor::block_on;
use mysql::prelude::ToValue;
use mysql::Value;
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
        let mut mp = MessPacketPt::new();
        mp.merge_from_bytes(&mess[..]);
        let mut gm = self.gm.clone();
        async_std::task::spawn(handler_mess_s(gm, mp));
    }
}

async fn handler_mess_s(gm: Arc<RwLock<GameMgr>>, mut mp: MessPacketPt) {
    //如果为空，什么都不执行
    if mp.get_cmd() != GameCode::Login as u32
        && mp.get_cmd() != GameCode::LineOff as u32
        && mp.get_data().is_empty()
    {
        error!("packet bytes is null!");
        return;
    }
    //判断是否执行登录
    if mp.get_cmd() == GameCode::Login as u32 {
        let mut c_login = C_USER_LOGIN_PROTO::new();
        let result = c_login.merge_from_bytes(mp.get_data());
        if result.is_err() {
            error!("{:?}", result.err().unwrap());
            return;
        }
        mp.set_user_id(c_login.userId);
        //执行登录
        login(gm, mp);
    } else {
        //不登录就执行其他命令
        gm.write().unwrap().invok(mp);
    }
}

//登录函数，执行登录
fn login(gm: Arc<RwLock<GameMgr>>, mut mess: MessPacketPt) {
    //玩家id
    let user_id = mess.user_id;
    let mut user_data = false;
    {
        user_data = gm.read().unwrap().users.contains_key(&user_id);
    }
    //走登录流程
    let mut gm_lock = gm.write().unwrap();
    //如果内存没有数据，则从数据库里面找
    if !user_data {
        let mut user = User::query(USER, user_id, None);
        //数据库没有则创建新号
        if user.is_none() {
            user = Some(User::new(user_id, "test", "test"));
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
    //返回客户端
    let mut lr = user2proto(user);
    let bytes = lr.write_to_bytes().unwrap();
    mess.set_user_id(user_id);
    mess.is_client = true;
    mess.cmd = ClientCode::Login as u32;
    mess.set_data(bytes);
    info!("用户完成登录！user_id:{}", &user_id);
    let result = gm_lock
        .sender
        .as_mut()
        .unwrap()
        .write(mess.write_to_bytes().unwrap());
    info!("发送给客户端!");
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
    lr.set_isSucc(true);
    // let result = user.get_json_value("signInTime");
    // if result.is_some() {
    //     let str = result.unwrap().as_str().unwrap();
    //     let mut sign_in_Time = str.parse::<NaiveDateTime>();
    //     lr.signInTime = sign_in_Time.unwrap().timestamp_subsec_micros();
    // }

    let mut result = user.get_time("syncTime");
    let mut time = 0 as u32;
    if result.is_some() {
        time = result.unwrap().timestamp_subsec_micros();
    }
    lr.syncTime = time;
    let mut ppt = PlayerPt::new();
    ppt.set_nick_name(
        user.get_json_value(NICK_NAME)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
    );
    ppt.dlc.push(1);
    lr.playerPt = protobuf::SingularPtrField::some(ppt);
    result = user.get_time("lastLoginTime");
    time = 0;
    if result.is_some() {
        time = result.unwrap().timestamp_subsec_micros();
    }

    lr.lastLoginTime = time;

    time = 0;
    result = user.get_time("lastLogOffTime");
    if result.is_some() {
        time = result.unwrap().timestamp_subsec_micros();
    }

    lr.lastLogOffTime = time;
    let mut res = ResourcesPt::new();
    res.id = 1;
    res.field_type = 1;
    res.num = 100 as u32;
    let mut v = Vec::new();
    v.push(res);
    let resp = protobuf::RepeatedField::from(v);
    lr.resp = resp;
    lr
}

pub fn new(address: &str, gm: Arc<RwLock<GameMgr>>) {
    let sh = TcpServerHandler {
        sender: None,
        gm: gm,
    };
    tools::tcp::tcp_server::new(address, sh);
}
///byte数组转换Packet
pub fn build_packet_mess_pt(mess: &MessPacketPt) -> Packet {
    //封装成packet
    let mut packet = Packet::new(mess.cmd);
    packet.set_data(&mess.write_to_bytes().unwrap()[..]);
    packet
}

///byte数组转换Packet
pub fn build_packet_bytes(bytes: &[u8]) -> Packet {
    let mut mpp = MessPacketPt::new();
    mpp.merge_from_bytes(bytes);

    //封装成packet
    let mut packet = Packet::new(mpp.cmd);
    packet.set_data(&mpp.write_to_bytes().unwrap()[..]);
    packet
}

///byte数组转换Packet
pub fn build_packet(mess: MessPacketPt) -> Packet {
    //封装成packet
    let mut packet = Packet::new(mess.cmd);
    packet.set_data(&mess.write_to_bytes().unwrap()[..]);
    packet
}
