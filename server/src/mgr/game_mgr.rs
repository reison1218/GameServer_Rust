use super::*;
use crate::protos::message::MsgEnum_MsgCode::C_USER_LOGIN;
use std::borrow::BorrowMut;
use std::sync::mpsc::{channel, Sender, SyncSender};
use tcp::tcp::MySyncSender;
use tcp::util::packet::PacketDes;

///gameMgr结构体
pub struct GameMgr {
    pub players: HashMap<u32, User>, //玩家数据
    pub pool: DbPool,                //db连接池
    pub sender: Option<MySyncSender>,
    pub cmd_map: HashMap<u32, fn(&mut GameMgr, Packet), RandomState>, //命令管理
}

impl GameMgr {
    ///创建gamemgr结构体
    pub fn new(pool: DbPool) -> GameMgr {
        let mut players: HashMap<u32, User> = HashMap::new();
        let mut gm = GameMgr {
            players: players,
            pool: pool,
            sender: None,
            cmd_map: HashMap::new(),
        };
        //初始化命令
        gm.cmd_init();
        gm
    }

    ///保存玩家数据
    pub fn save_user(&mut self) {
        info!("执行保存，保存所有内存玩家数据");

        let time = std::time::SystemTime::now();
        let mut pool = &mut self.pool;
        let mut re: Option<Result<u32, String>> = None;
        let mut count: u32 = 0;
        for (k, mut v) in &mut self.players {
            if v.get_version() <= 0 {
                continue;
            }
            re = Some(v.update(pool));
            if re.is_some() {
                match re.unwrap() {
                    Err(str) => {
                        error!("玩家数据保存异常user_id：{}，message:{:?}", k, str);
                    }
                    _ => {
                        count += 1;
                    }
                }
            }
        }
        info!(
            "玩家数据保存结束，保存个数:{},耗时：{}ms",
            count,
            time.elapsed().unwrap().as_millis()
        );
    }

    ///执行函数，通过packet拿到cmd，然后从cmdmap拿到函数指针调用
    pub fn invok(&mut self, packet: Packet) {
        let cmd = packet.get_cmd();
        let f = self.cmd_map.get_mut(&cmd);
        if f.is_none() {
            return;
        }
        f.unwrap()(self, packet);
    }

    ///命令初始化
    fn cmd_init(&mut self) {
        self.cmd_map.insert(SYNC, sync);
        self.cmd_map.insert(OFF_LINE, off_line);
    }
}

///同步数据
fn sync(gm: &mut GameMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let user = gm.players.get_mut(&user_id.unwrap());
    if user.is_none() {
        error!("user data is null for id:{}", user_id.unwrap());
        return;
    }
    let user = user.unwrap();

    info!("执行同步函数");
}

///玩家离线
fn off_line(gm: &mut GameMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let user = gm.players.remove(&user_id.unwrap());
    if user.is_some() {
        let user = user.unwrap();
        info!("游戏服已处理玩家离线 for id:{}", user.user_id);
    }
}
