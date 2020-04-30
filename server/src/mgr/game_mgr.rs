use super::*;
use crate::entity::user::UserData;
use crate::entity::EntityData;
use crate::redispool::redistool;
use crate::redispool::redistool::RedisPoolTool;
use crate::DB_POOL;
use chrono::{Local, Timelike};
use protobuf::well_known_types::Any;
use protobuf::Message;
use std::borrow::BorrowMut;
use std::convert::TryFrom;
use std::sync::mpsc::{channel, Sender, SyncSender};
use std::time::Duration;
use tools::cmd_code::ClientCode;
use tools::protos::base::MessPacketPt;
use tools::protos::protocol::S_SYNC_DATA;
use tools::tcp::TcpSender;
use tools::util::packet::PacketDes;

///gameMgr结构体
pub struct GameMgr {
    pub users: HashMap<u32, UserData>, //玩家数据
    pub sender: Option<TcpSender>,     //tcpchannel
    pub cmd_map: HashMap<u32, fn(&mut GameMgr, MessPacketPt), RandomState>, //命令管理
}

impl GameMgr {
    ///创建gamemgr结构体
    pub fn new() -> GameMgr {
        let mut users: HashMap<u32, UserData> = HashMap::new();
        let mut gm = GameMgr {
            users,
            sender: None,
            cmd_map: HashMap::new(),
        };
        //初始化命令
        gm.cmd_init();
        gm
    }

    pub fn save_user_http(&mut self) {
        let time = std::time::SystemTime::now();
        let mut count: u32 = 0;
        for (k, mut v) in self.users.iter_mut() {
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
    pub fn save_user(&mut self, sender: Sender<Vec<Box<dyn EntityData>>>) {
        let time = std::time::SystemTime::now();
        let mut v: Vec<Box<dyn EntityData>> = Vec::new();
        for ud in self.users.values_mut() {
            if ud.get_version() <= 0 {
                continue;
            };
            ud.clear_version();
            v.push(ud.get_user_info_ref().try_clone());
        }
        let count = v.len();
        if count > 0 {
            sender.send(v);
        }
        info!(
            "开始执行定时保存玩家，发送数量:{},耗时:{}ms",
            count,
            time.elapsed().unwrap().as_millis()
        );
    }

    ///执行函数，通过packet拿到cmd，然后从cmdmap拿到函数指针调用
    pub fn invok(&mut self, packet: MessPacketPt) {
        let cmd = packet.get_cmd();
        let f = self.cmd_map.get_mut(&cmd);
        if f.is_none() {
            return;
        }
        f.unwrap()(self, packet);
    }

    ///命令初始化
    fn cmd_init(&mut self) {
        self.cmd_map.insert(SyncData as u32, sync);
        self.cmd_map.insert(LineOff as u32, off_line);
    }
}

///同步数据
fn sync(gm: &mut GameMgr, mut mess: MessPacketPt) {
    let user_id = mess.get_user_id();
    let user = gm.users.get_mut(&user_id);
    if user.is_none() {
        error!("user data is null for id:{}", user_id);
        return;
    }
    mess.is_client = true;
    mess.cmd = ClientCode::SyncData as u32;
    let mut s_s_d = S_SYNC_DATA::new();
    s_s_d.isSucc = true;
    s_s_d.syncTime = Local::now().naive_local().timestamp_subsec_micros();
    mess.set_data(s_s_d.write_to_bytes().unwrap());
    gm.sender
        .as_mut()
        .unwrap()
        .write(mess.write_to_bytes().unwrap());
    info!("执行同步函数");
}

///玩家离线
fn off_line(gm: &mut GameMgr, packet: MessPacketPt) {
    let user_id = packet.get_user_id();
    let user = gm.users.remove(&user_id);
    if user.is_some() {
        let mut user = user.unwrap();
        user.update();
        info!("游戏服已处理玩家离线 for id:{}", user.get_user_id());
    }
}
