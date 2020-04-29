use super::*;
use crate::entity::member::{Member, MemberState, Target, UserType};
use tools::protos::base::MessPacketPt;
use tools::thread_pool::ThreadPoolType::user;

pub struct RoomCache {
    room_id: u64,
    count: u32,
}

pub struct RoomMgr {
    pub player_room: HashMap<u32, u64>, //key:玩家id    value:房间id
    pub rooms: HashMap<u64, Room>,      //key:房间id    value:房间结构体
    pub room_cache: Vec<RoomCache>,     //key:房间id    value:房间人数
    pub sender: Option<TcpSender>,
    pub cmd_map: HashMap<u32, fn(&mut RoomMgr, MessPacketPt), RandomState>, //命令管理
}

impl RoomMgr {
    pub fn new() -> RoomMgr {
        let player_room: HashMap<u32, u64> = HashMap::new();
        let rooms: HashMap<u64, Room> = HashMap::new();
        let cmd_map: HashMap<u32, fn(&mut RoomMgr, MessPacketPt), RandomState> = HashMap::new();
        let room_cache: Vec<RoomCache> = Vec::new();
        let mut rm = RoomMgr {
            player_room,
            rooms,
            room_cache,
            sender: None,
            cmd_map,
        };
        rm.cmd_init();
        rm
    }

    ///检查玩家是否已经在房间里
    pub fn check_player(&self, user_id: &u32) -> bool {
        self.player_room.contains_key(user_id)
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

    ///删除缓存房间
    fn remove_room_cache(&mut self, room_id: &u64) {
        let mut index: usize = 0;
        for i in self.room_cache.iter() {
            index += 1;
            if i.room_id != *room_id {
                continue;
            }
            break;
        }
        self.room_cache.remove(index);
    }
}

///创建房间
fn create_room(rm: &mut RoomMgr, packet: MessPacketPt) {
    let user_id = packet.get_user_id();
    let room_id = rm.player_room.get_mut(&user_id);
    if room_id.is_some() {
        error!("user data is null for id:{}", user_id);
        return;
    }
}

///离开房间
fn leave_room(rm: &mut RoomMgr, packet: MessPacketPt) {
    let user_id = packet.user_id;
    let mut res = rm.player_room.get_mut(&user_id);
    if res.is_none() {
        return;
    }
    let room_id = res.unwrap();
    let room_id = *room_id;
    let room = rm.rooms.get_mut(&room_id);
    if room.is_none() {
        return;
    }
    let mut room = room.unwrap();
    room.remove_member(&user_id);
    rm.player_room.remove(&user_id);
    rm.room_cache.retain(|x| x.room_id != room_id);
}

///改变目标
fn change_target(rm: &mut RoomMgr, packet: MessPacketPt) {}

///寻找房间并加入房间
fn search_room(rm: &mut RoomMgr, packet: MessPacketPt) {
    let result = rm.room_cache.last_mut();
    let mut need_remove = false;
    let mut add_res = false;
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
                    rm.player_room.insert(packet.user_id, room.get_room_id());
                    add_res = true;
                    if rc.count >= 4 {
                        need_remove = true;
                    }
                }
                None => {}
            }
        }
        None => {}
    }
    rm.room_cache
        .sort_by(|a, b| a.count.partial_cmp(&b.count).unwrap());
    if !need_remove {
        return;
    }
    rm.room_cache.pop();
}

///准备
fn prepare_cancel(rm: &mut RoomMgr, packet: MessPacketPt) {
    //校验玩家是否在房间
    let res = rm.player_room.contains_key(&packet.user_id);
    if !res {
        return;
    }
}

///开始
fn start(rm: &mut RoomMgr, packet: MessPacketPt) {
    let room = check_player_in_room(&packet.user_id, rm);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();
    let res = room.check_ready();
    if !res {
        return;
    }
    let room_id = room.get_room_id();
    rm.remove_room_cache(&room_id);
}

///换队伍
fn change_team(rm: &mut RoomMgr, packet: MessPacketPt) {
    let room = check_player_in_room(&packet.user_id, rm);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();
    room.change_team(&packet.user_id, &(0 as u8));
}
///T人
fn kick_member(rm: &mut RoomMgr, packet: MessPacketPt) {
    let room = check_player_in_room(&packet.user_id, rm);
    if room.is_none() {
        return;
    }
    let room = room.unwrap();
    let taret_user: u32 = 0;
    let res = room.kick_member(&packet.user_id, &taret_user);
    if res.is_err() {
        res.unwrap_err();
        return;
    }
}

///检查玩家是否在房间里
fn check_player_in_room<'a, 'b: 'a>(user_id: &'b u32, rm: &'b mut RoomMgr) -> Option<&'a mut Room> {
    let room_id = rm.player_room.get(user_id);
    if room_id.is_none() {
        return None;
    }
    let room_id = room_id.unwrap();
    let room = rm.rooms.get_mut(room_id);
    if room.is_none() {
        return None;
    }
    room
}
