use super::*;
use crate::entity::battle_model::{FriendRoom, PVPModel, PubRoom, RoomModel};
use crate::entity::member::{Member, MemberState, Target, UserType};
use crate::template::template_contants::TILE_MAP_TEMPLATE;
use crate::template::templates::Template;
use crate::TEMPLATES;
use protobuf::Message;
use serde_json::{Map, Value};
use tools::cmd_code::ClientCode;
use tools::protos::base::{RoomPt, TeamPt};
use tools::protos::room::S_ROOM;
use tools::thread_pool::ThreadPoolType::user;
use tools::util::packet::Packet;

pub struct RoomMgr {
    pub friend_room: FriendRoom,         //好友房
    pub pub_rooms: HashMap<u8, PubRoom>, //公共房
    pub sender: Option<TcpSender>,
    pub cmd_map: HashMap<u32, fn(&mut RoomMgr, Packet), RandomState>, //命令管理
}

impl RoomMgr {
    pub fn new() -> RoomMgr {
        let cmd_map: HashMap<u32, fn(&mut RoomMgr, Packet), RandomState> = HashMap::new();
        let friend_room = FriendRoom::default();
        let pub_rooms: HashMap<u8, PubRoom> = HashMap::new();
        let mut rm = RoomMgr {
            friend_room,
            pub_rooms,
            sender: None,
            cmd_map,
        };
        rm.cmd_init();
        rm
    }

    ///检查玩家是否已经在房间里
    pub fn check_player(&self, user_id: &u32) -> bool {
        self.friend_room.check_is_in_room(user_id)
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
        self.cmd_map
            .insert(RoomCode::CreateRoom as u32, create_room);
        self.cmd_map.insert(RoomCode::LeaveRoom as u32, leave_room);
    }
}

///创建房间
fn create_room(rm: &mut RoomMgr, mut packet: Packet) {
    let user_id = packet.get_user_id();
    //校验这个用户在不在房间内
    let in_room = rm.friend_room.check_is_in_room(&user_id);
    if in_room {
        error!("user data is null for id:{}", user_id);
        return;
    }
    //解析protobuf
    let mut cr = tools::protos::room::C_CREATE_ROOM::new();
    let result = cr.merge_from_bytes(packet.get_data());
    if result.is_err() {
        error!("{:?}", result.err().unwrap());
        return;
    }

    let map_id = cr.map_id as u64;
    //校验地图配置
    let res: Option<&Template> = TEMPLATES.get(TILE_MAP_TEMPLATE, &map_id);
    if res.is_none() {
        error!("this map config is None,map_id:{}", map_id);
        return;
    }
    //创建房间
    let map_temp = res.unwrap();
    let res = rm.friend_room.create_room(&user_id, map_temp);
    if res.is_err() {
        return;
    }

    //组装protobuf
    let room = res.unwrap();
    let mut s_r = S_ROOM::new();
    s_r.is_succ = true;
    let rp = room.convert_to_pt();
    s_r.set_room(rp);

    //封装客户端端消息包，并返回客户端
    packet.set_user_id(user_id);
    packet.set_is_client(true);
    packet.set_cmd(ClientCode::Room as u32);
    packet.set_data_from_vec(s_r.write_to_bytes().unwrap());
    let v = packet.build_server_bytes();
    rm.sender.as_mut().unwrap().write(v);
}

///离开房间
fn leave_room(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();

    rm.friend_room.leave_room(&user_id);

    info!("已处理离线玩家！id:{}", packet.get_user_id());
}

///改变目标
fn change_target(rm: &mut RoomMgr, packet: Packet) {}

///寻找房间并加入房间
fn search_room(rm: &mut RoomMgr, packet: Packet) {
    let room_model = 1 as u8;
    let user_id = packet.get_user_id();
    let result = rm.pub_rooms.get_mut(&room_model);
    if result.is_none() {
        error!("this model is not exist!model_type:{}", room_model);
        return;
    }
    let mut pub_room = result.unwrap();
    pub_room.quickly_start(&user_id);
}

///准备
fn prepare_cancel(rm: &mut RoomMgr, packet: Packet) {
    //校验玩家是否在房间
    // let res = rm.player_room.contains_key(&packet.get_user_id());
    // if !res {
    //     return;
    // }
}

///开始
fn start(rm: &mut RoomMgr, packet: Packet) {
    // let user_id = &packet.get_user_id();
    // let room = check_player_in_room(user_id, rm);
    // if room.is_none() {
    //     return;
    // }
    // let room = room.unwrap();
    // let res = room.check_ready();
    // if !res {
    //     return;
    // }
    // let room_id = room.get_room_id();
    // rm.remove_room_cache(&room_id);
}

///换队伍
fn change_team(rm: &mut RoomMgr, packet: Packet) {
    // let user_id = &packet.get_user_id();
    // let room = check_player_in_room(user_id, rm);
    // if room.is_none() {
    //     return;
    // }
    // let room = room.unwrap();
    // room.change_team(&packet.get_user_id(), &(0 as u8));
}
///T人
fn kick_member(rm: &mut RoomMgr, packet: Packet) {
    // let user_id = &packet.get_user_id();
    // let room = check_player_in_room(user_id, rm);
    // if room.is_none() {
    //     return;
    // }
    // let room = room.unwrap();
    // let taret_user: u32 = 0;
    // let res = room.kick_member(&packet.get_user_id(), &taret_user);
    // if res.is_err() {
    //     res.unwrap_err();
    //     return;
    // }
}
