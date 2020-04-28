use super::*;
use crate::entity::member::{Member, MemberState, Target, UserType};
use tools::protos::base::MessPacketPt;

pub struct RoomCache {
    room_id: u64,
    count: u32,
}

pub struct RoomMgr {
    pub players: HashMap<u32, u64>,  //key:玩家id    value:房间id
    pub rooms: HashMap<u64, Room>,   //key:房间id    value:房间结构体
    pub room_member: Vec<RoomCache>, //key:房间id    value:房间人数
    pub sender: Option<TcpSender>,
    pub cmd_map: HashMap<u32, fn(&mut RoomMgr, MessPacketPt), RandomState>, //命令管理
}

impl RoomMgr {
    pub fn new() -> RoomMgr {
        let players: HashMap<u32, u64> = HashMap::new();
        let rooms: HashMap<u64, Room> = HashMap::new();
        let cmd_map: HashMap<u32, fn(&mut RoomMgr, MessPacketPt), RandomState> = HashMap::new();
        let room_member: Vec<RoomCache> = Vec::new();
        let mut rm = RoomMgr {
            players,
            rooms,
            room_member,
            sender: None,
            cmd_map,
        };
        rm.cmd_init();
        rm
    }

    ///检查玩家是否已经在房间里
    pub fn check_player(&self, user_id: &u32) -> bool {
        self.players.contains_key(user_id)
    }

    ///检查玩家准备状态
    pub fn check_ready(&self, room_id: &u64) -> bool {
        let res = self.rooms.contains_key(room_id);
        if !res {
            return false;
        }
        let res = self.rooms.get(room_id).unwrap();

        res.check_ready()
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
        self.cmd_map
            .insert(RoomCode::CreateRoom as u32, create_room);
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

///离开房间
fn leave_room(rm: &mut RoomMgr, packet: MessPacketPt) {}

///改变目标
fn change_target(rm: &mut RoomMgr, packet: MessPacketPt) {}

///寻找房间并加入房间
fn search_room(rm: &mut RoomMgr, packet: MessPacketPt) {
    let result = rm.room_member.first_mut();
    match result {
        Some(rc) => {
            let room_id = rc.room_id;
            let mut room = rm.rooms.get_mut(&room_id);
            match room {
                Some(room) => {
                    let mut member = Member {
                        user_id: packet.user_id,
                        user_type: UserType::Real as u8,
                        state: MemberState::NotReady as u8,
                        target: Target::default(),
                    };
                    room.add_member(member);
                    rc.count += 1;
                    rm.players.insert(packet.user_id, room.get_room_id());
                }
                None => {}
            }
        }
        None => {}
    }
}

///准备
fn prepare_cancel(rm: &mut RoomMgr, packet: MessPacketPt) {
    let user_id = packet.user_id;
    //校验玩家是否在房间
    let res = rm.players.contains_key(&user_id);
    if !res {
        return;
    }
}

///开始
fn start(rm: &mut RoomMgr, packet: MessPacketPt) {
    //rm.check_ready(1 a);
}
