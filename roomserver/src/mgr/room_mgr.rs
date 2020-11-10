use crate::handlers::handler::do_something;
use log::warn;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use tools::cmd_code::{ClientCode, RoomCode};
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

type CmdFn = HashMap<u32, fn(&mut RoomMgr, Packet) -> anyhow::Result<()>, RandomState>;

///房间服管理器
pub struct RoomMgr {
    pub player_room: HashMap<u32, u64>, //玩家对应的房间，key:u32,value:采用一个u64存，通过位运算分出高低位,低32位是房间模式,高32位是房间id
    pub cmd_map: CmdFn,                 //命令管理 key:cmd,value:函数指针
    sender: Option<TcpSender>,          //tcp channel的发送方
}

tools::get_mut_ref!(RoomMgr);

impl RoomMgr {
    pub fn new() -> RoomMgr {
        let cmd_map: HashMap<u32, fn(&mut RoomMgr, Packet) -> anyhow::Result<()>, RandomState> =
            HashMap::new();
        let player_room: HashMap<u32, u64> = HashMap::new();
        let mut rm = RoomMgr {
            player_room,
            sender: None,
            cmd_map,
        };
        rm.cmd_init();
        rm
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd.into_u32(), user_id, bytes, true, true);
        self.get_sender_mut().write(bytes);
    }

    pub fn set_sender(&mut self, sender: TcpSender) {
        self.sender = Some(sender);
    }

    pub fn get_sender_clone(&self) -> TcpSender {
        self.sender.clone().unwrap()
    }

    pub fn get_sender_mut(&mut self) -> &mut TcpSender {
        self.sender.as_mut().unwrap()
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

    ///命令初始化
    fn cmd_init(&mut self) {
        //dosomething
        self.cmd_map
            .insert(RoomCode::DoSomeThing.into_u32(), do_something);
    }
}
