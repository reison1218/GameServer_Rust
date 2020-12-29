use log::warn;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::net::TcpStream;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

///命令别名
type CmdFn = HashMap<u32, fn(&mut GameCenterMgr, Packet) -> anyhow::Result<()>, RandomState>;

pub struct GameCenterMgr {
    pub cmd_map: CmdFn,                     //命令管理 key:cmd,value:函数指针
    pub room_mgr: Option<TcpStream>,        //房间管理服
    pub gates: HashMap<usize, TcpSender>,   //gate路由服客户端
    pub rooms: HashMap<usize, TcpSender>,   //房间服客户端
    pub player_w_gate: HashMap<u32, usize>, //玩家对应gate
}

impl GameCenterMgr {
    pub fn add_client(&mut self, sender: TcpSender) {
        let key = sender.token;
        self.tcp_clients.insert(key, sender);
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
