use super::*;
use tools::protos::base::MessPacketPt;


pub struct RoomMgr{
    pub players:HashMap<u32,u32>,//key:玩家id    value:房间id
    pub rooms:HashMap<u32,Room>,//key:房间id    value:房间结构体
    pub sender:Option<TcpSender>,
    pub cmd_map: HashMap<u32, fn(&mut RoomMgr, MessPacketPt), RandomState>, //命令管理
}

impl RoomMgr {
    pub fn new()->RoomMgr{
        let players:HashMap<u32,u32> = HashMap::new();
        let rooms:HashMap<u32,Room> = HashMap::new();
        let cmd_map:HashMap<u32, fn(&mut RoomMgr, MessPacketPt), RandomState> = HashMap::new();
        let mut rm = RoomMgr{players,rooms,sender:None,cmd_map};
        rm.cmd_init();
        rm
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
        self.cmd_map.insert(RoomCode::CreateRoom as u32, create_room);
        self.cmd_map.insert(RoomCode::LeaveRoom as u32, leave_room);
    }
}

///创建房间
fn create_room(rm: &mut RoomMgr, packet: MessPacketPt) {
    let user_id = packet.get_user_id();
    let user = rm.players.get_mut(&user_id);
    if user.is_none() {
        error!("user data is null for id:{}", user_id);
        return;
    }
    info!("执行同步函数");
}

fn leave_room(rm: &mut RoomMgr, packet: MessPacketPt){

}