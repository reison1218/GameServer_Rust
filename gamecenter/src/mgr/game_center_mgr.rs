use log::warn;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::net::TcpStream;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

///命令别名
type CmdFn = HashMap<u32, fn(&mut GameCenterMgr, Packet) -> anyhow::Result<()>, RandomState>;

#[derive(Default)]
pub struct GameCenterMgr {
    pub cmd_map: CmdFn,                             //命令管理 key:cmd,value:函数指针
    pub room_center: Option<TcpStream>,             //房间中心
    pub gate_clients: HashMap<usize, GateClient>,   //gate路由服客户端
    pub battle_clients: HashMap<usize, RoomClient>, //战斗服客户端
    pub player_w_gate: HashMap<u32, usize>,         //玩家对应gate
}

pub struct GateClient {
    token: usize,
    sender: TcpSender,
}

impl GateClient {
    pub fn new(token: usize, sender: TcpSender) -> Self {
        let gc = GateClient { token, sender };
        gc
    }
}

pub struct RoomClient {
    token: usize,
    sender: TcpSender,
    room_num: u32,
}

impl RoomClient {
    pub fn new(token: usize, sender: TcpSender) -> Self {
        let rc = RoomClient {
            token,
            sender,
            room_num: 0,
        };
        rc
    }
}

impl GameCenterMgr {
    pub fn new() -> Self {
        GameCenterMgr::default()
    }
    pub fn add_gate_client(&mut self, sender: TcpSender) {
        let key = sender.token;
        let gc = GateClient::new(sender.token, sender);
        self.gate_clients.insert(key, gc);
    }

    pub fn add_battle_client(&mut self, sender: TcpSender) {
        let key = sender.token;
        let rc = RoomClient::new(sender.token, sender);
        self.battle_clients.insert(key, rc);
    }

    ///执行函数，通过packet拿到cmd，然后从cmdmap拿到函数指针调用
    pub fn invok(&mut self, packet: Packet) {
        let cmd = packet.get_cmd();
        let f = self.cmd_map.get_mut(&cmd);
        if f.is_none() {
            warn!("there is no handler of cmd:{:?}!", cmd);
            return;
        }
        let _ = f.unwrap()(self, packet);
    }
}
