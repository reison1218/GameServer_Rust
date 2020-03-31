use super::*;
use chrono::{Duration, Local, NaiveDateTime, NaiveTime};
use std::io::Write;
use tcp::thread_pool::ThreadPoolHandler;

use crate::protos::base::{MessPacketPt, ResourcesPt};
use tcp::tcp::{tcp_server, MySyncSender};

use crate::entity::Entity;
use crate::protos::message::MsgEnum_MsgCode::LOG_OFF;
use crate::prototools::proto;
use futures::executor::block_on;
use mysql::prelude::ToValue;
use mysql::Value;
use std::str::FromStr;
use std::sync::mpsc::{Sender, SyncSender};

#[derive(Clone)]
struct ServerHandler {
    sender: Option<MySyncSender>,
    gm: Arc<RwLock<GameMgr>>,
}

unsafe impl Send for ServerHandler {}

unsafe impl Sync for ServerHandler {}

impl tcp::tcp::Handler for ServerHandler {
    fn on_open(&mut self, sender: MySyncSender) {
        self.gm.write().unwrap().sender = Some(sender);
    }

    fn on_close(&mut self) {
        info!("与tcp客户端断开连接");
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        self.handler_mess(mess);
    }
}

impl ServerHandler {
    fn handler_mess(&mut self, mess: Vec<u8>) {
        let mut bb = ByteBuf::form_vec(mess);
        let mut packet = Packet::from(bb);
        let mut gm = self.gm.clone();
        //异步处理
        let async_code = async {
            //如果为空，什么都不执行
            if packet.get_cmd() != LOG_OFF.value() as u32 && packet.get_data().is_empty() {
                info!("packet bytes is null!");
                return;
            }
            //判断是否执行登录
            if packet.get_cmd() == C_USER_LOGIN.value() as u32 {
                let mut c_login = C_USER_LOGIN_PROTO::new();
                let result = c_login.merge_from_bytes(packet.get_data());
                if result.is_err() {
                    error!("{:?}", result.err().unwrap());
                    return;
                }
                packet.set_user_id(c_login.userId);
                //执行登录
                login(gm, packet);
            } else {
                //不登录就执行其他命令
                gm.write().unwrap().invok(packet);
            }
        };
        //交给异步处理执行器执行
        async_std::task::spawn(async {
            async_code.await;
        });
    }
}

//登录函数，执行登录
fn login(gm: Arc<RwLock<GameMgr>>, mut packet: Packet) {
    //玩家id
    let user_id = packet.get_user_id().unwrap();

    let mut user_data = false;
    {
        user_data = gm.read().unwrap().players.contains_key(&user_id);
    }
    //校验玩家是否登录过

    //走登录流程
    let mut gm_lock = gm.write().unwrap();
    //如果内存没有数据，则从数据库里面找
    if !user_data {
        let mut user = User::query(user_id, &mut gm_lock.pool);
        //数据库没有则创建新号
        if user.is_none() {
            user = Some(User::new(user_id, "test", "test"));
            //以下入库采用异步执行，以免造成io堵塞
            let gm_clone = gm.clone();
            let mut user_mut = user.clone().unwrap();
            let m = move || {
                let mut gm_lock = gm_clone.write().unwrap();
                info!("玩家数据不存在,现在创建新玩家:{}", user_id);
                let result = User::insert(&mut user_mut, &mut gm_lock.pool);
                if result.is_err() {
                    error!("{:?}", result.err().unwrap());
                }
            };
            &THREAD_POOL.submit_game(m);
        }
        //封装到内存中
        gm_lock.players.insert(user_id, user.unwrap());
    }
    //封装会话
    let user = gm_lock.players.get_mut(&user_id);
    if user.is_none() {
        error!("there is no data for userid:{}", &user_id);
        return;
    }
    let user = user.unwrap();
    user.update_login_time();
    //返回客户端
    let mut lr = user2proto(user);
    let bytes = lr.write_to_bytes().unwrap();
    let mut packet = Packet::new(5003 as u32);
    packet.set_user_id(user_id);
    packet.set_data(&lr.write_to_bytes().unwrap()[..]);
    info!("用户完成登录！user_id:{}", &user_id);
    let result = gm_lock.sender.as_mut().unwrap().sender.send(packet);
    info!("发送rec端!");
    if result.is_err() {
        error!("{:?}", result.unwrap_err());
    }
}

///user结构体转proto
fn user2proto(user: &mut User) -> S_USER_LOGIN_PROTO {
    let mut lr = S_USER_LOGIN_PROTO::new();
    lr.set_isSucc(true);
    lr.userId = user.user_id;
    lr.avatar = user.get_str(AVATAR).unwrap().to_owned();
    lr.nickName = user.get_str(NICK_NAME).unwrap().to_owned();
    let signIn = user.get_usize("signIn");
    if signIn.is_some() {
        lr.signIn = signIn.unwrap() as u32;
    }

    let result = user.get_json_value("signInTime");
    if result.is_some() {
        let str = result.unwrap().as_str().unwrap();
        let mut signInTime = str.parse::<NaiveDateTime>();
        lr.signInTime = signInTime.unwrap().timestamp_subsec_micros();
    }
    let battlepos_id = user.get_usize("battlePosId");
    if battlepos_id.is_some() {
        lr.battlePosId = battlepos_id.unwrap() as u32;
    }

    lr.offLineGold = user.get_usize("offLineGold").unwrap() as f64;
    let dayRankReward = user.get_usize("dayRankReward");
    if dayRankReward.is_some() {
        lr.dayRankReward = dayRankReward.unwrap() as u32;
    }

    let dayRank = user.get_usize("dayRank");
    if dayRank.is_some() {
        lr.dayRank = dayRank.unwrap() as u32;
    }
    let turnCount = user.get_usize("turnCount");
    if turnCount.is_some() {
        lr.turnCount = turnCount.unwrap() as u32;
    }
    let mut result = user.get_time("syncTime");
    let mut time = 0 as u32;
    if result.is_some() {
        time = result.unwrap().timestamp_subsec_micros();
    }
    lr.syncTime = time;
    let mut ppt = PlayerPt::new();
    ppt.maxcp = user.get_usize(MAX_CP).unwrap() as u32;
    ppt.maxJumpLevel = user.get_usize(MAX_JUMP_LEVEL).unwrap() as u32;
    ppt.maxMultiple = user.get_usize(MAX_MULTIPLE).unwrap() as u32;
    ppt.maxJumpRange = user.get_usize(MAX_JUMP_RANGE).unwrap() as u32;
    ppt.maxScore = user.get_usize(MAX_SCORE).unwrap() as u32;
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
    lr.tujianHeatBallId = 1;
    let mut v = Vec::new();
    v.push(1 as u32);
    lr.specialId = v;
    let mut res = ResourcesPt::new();
    res.id = 1;
    res.field_type = 1;
    res.num = 100 as f64;
    let mut v = Vec::new();
    v.push(res);
    let resp = protobuf::RepeatedField::from(v);
    lr.resp = resp;
    lr
}

pub fn new(address: &str, gm: Arc<RwLock<GameMgr>>) {
    let sh = ServerHandler {
        sender: None,
        gm: gm,
    };
    let mut tcp_server = tcp_server::new(address, sh).unwrap();
    tcp_server.on_listen();
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
