use super::*;
use crate::entity::battle_model::{FriendRoom, PVPModel, PubRoom, RoomModel};
use crate::entity::member::{Member, MemberState, UserType};
use crate::TEMPLATES;
use log::{error, info, warn};
use protobuf::Message;
use tools::cmd_code::ClientCode;
use tools::protos::base::RoomPt;
use tools::protos::room::{C_JOIN_ROOM, S_ROOM};
use tools::protos::server_protocol::{
    PlayerBattlePt, G_R_CREATE_ROOM, G_R_JOIN_ROOM, G_R_SEARCH_ROOM,
};
use tools::templates::tile_map_temp::TileMapTempMgr;
use tools::util::packet::Packet;

pub struct RoomMgr {
    pub friend_room: FriendRoom,         //好友房
    pub pub_rooms: HashMap<u8, PubRoom>, //公共房
    pub sender: Option<TcpSender>,
    pub cmd_map: HashMap<u32, fn(&mut RoomMgr, Packet) -> anyhow::Result<()>, RandomState>, //命令管理
}

impl RoomMgr {
    pub fn new() -> RoomMgr {
        let cmd_map: HashMap<u32, fn(&mut RoomMgr, Packet) -> anyhow::Result<()>, RandomState> =
            HashMap::new();
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
        let res = self.friend_room.check_is_in_room(user_id);
        if res {
            return true;
        }
        for room in self.pub_rooms.iter() {
            let res = room.1.check_is_in_room(user_id);
            if res {
                return true;
            }
        }
        false
    }

    ///执行函数，通过packet拿到cmd，然后从cmdmap拿到函数指针调用
    pub fn invok(&mut self, packet: Packet) {
        let cmd = packet.get_cmd();
        let f = self.cmd_map.get_mut(&cmd);
        if f.is_none() {
            warn!("there is no handler of cmd:{:?}!", cmd);
            return;
        }
        let res: anyhow::Result<()> = f.unwrap()(self, packet);
        match res {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    ///命令初始化
    fn cmd_init(&mut self) {
        self.cmd_map
            .insert(RoomCode::CreateRoom as u32, create_room);
        self.cmd_map.insert(RoomCode::LeaveRoom as u32, leave_room);
        self.cmd_map
            .insert(RoomCode::ChangeTeam as u32, change_team);
        self.cmd_map.insert(RoomCode::Kick as u32, kick_member);
        self.cmd_map.insert(RoomCode::StartGame as u32, start);
        self.cmd_map
            .insert(RoomCode::PrepareCancel as u32, prepare_cancel);
        self.cmd_map.insert(RoomCode::LineOff as u32, leave_room);
        self.cmd_map.insert(RoomCode::JoinRoom as u32, join_room);
        self.cmd_map
            .insert(RoomCode::SearchRoom as u32, search_room);
    }

    pub fn send(&mut self, bytes: Vec<u8>) {
        if self.sender.is_none() {
            error!("room_mgr'sender is None!");
            return;
        }
        self.sender.as_mut().unwrap().write(bytes);
    }
}

///创建房间
fn create_room(rm: &mut RoomMgr, mut packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    //校验这个用户在不在房间内
    let room_id = rm.friend_room.get_room_id_by_user_id(&user_id);
    if room_id.is_some() {
        let s = format!(
            "this user already in the room,can not create room! user_id:{},room_id:{}",
            user_id,
            room_id.unwrap()
        );
        warn!("{:?}", s.as_str());
        anyhow::bail!(s)
    }
    //解析protobuf
    let mut gr = G_R_CREATE_ROOM::new();
    gr.merge_from_bytes(packet.get_data())?;

    let map_id = gr.map_id;
    //校验地图配置
    let map_temp: &TileMapTempMgr = TEMPLATES.get_tile_map_ref();
    //创建房间
    let temp = map_temp.get_temp(map_id)?;
    let owner = Member::from(gr.take_pbp());

    let room = rm.friend_room.create_room(owner, temp)?;
    println!("room size:{}", std::mem::size_of_val(&room));
    //组装protobuf
    let mut s_r = S_ROOM::new();
    s_r.is_succ = true;
    let rp = room.convert_to_pt();
    println!("roomPt size:{}", std::mem::size_of_val(&rp));
    s_r.set_room(rp);
    let res = s_r.write_to_bytes().unwrap();
    println!("bytes size:{}", std::mem::size_of_val(&res));
    //封装客户端端消息包，并返回客户端
    packet.set_user_id(user_id);
    packet.set_is_client(true);
    packet.set_cmd(ClientCode::Room as u32);
    packet.set_data_from_vec(res);
    let v = packet.build_server_bytes();
    let res = rm.sender.as_mut().unwrap().write(v);
    if res.is_err() {
        let str = format!("{:?}", res.err().unwrap().to_string());
        error!("{:?}", str.as_str());
        anyhow::bail!("{:?}", str)
    }
    Ok(())
}

///离开房间
fn leave_room(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    //处理好友房
    let res = rm.friend_room.leave_room(&user_id);
    match res {
        Ok(_) => info!(
            "卸载玩家好友房数据！user_id:{},room_id:{}",
            user_id,
            res.unwrap()
        ),
        Err(_) => {}
    }
    //处理随机房
    let mut room_id = 0;
    for pub_room in rm.pub_rooms.iter_mut() {
        let res = pub_room.1.player_room.get(&user_id);
        if res.is_none() {
            continue;
        }
        room_id = pub_room.1.leave_room(&user_id)?;
    }
    if room_id > 0 {
        info!(
            "卸载玩家公共pvp房数据！user_id:{},room_id:{}",
            user_id, room_id
        );
    }
    Ok(())
}

///改变目标
fn change_target(rm: &mut RoomMgr, mut packet: Packet) -> anyhow::Result<()> {
    Ok(())
}

///寻找房间并加入房间
fn search_room(rm: &mut RoomMgr, mut packet: Packet) -> anyhow::Result<()> {
    let mut grs = G_R_SEARCH_ROOM::new();
    grs.merge_from_bytes(packet.get_data());

    let room_model = grs.get_model_type() as u8;
    let user_id = packet.get_user_id();
    ///校验模式
    if room_model < PVPModel::OneVOneVOneVOne as u8 || room_model > PVPModel::OneVOne as u8 {
        let s = format!("this model is not exist!model_type:{}", room_model);
        anyhow::bail!(s)
    }
    let result = rm.pub_rooms.get_mut(&room_model);
    if result.is_none() {
        //如果没有，则初始化公共房间
        let mut pr = PubRoom::default();
        pr.model_type = room_model as u8;
        rm.pub_rooms.insert(room_model, pr);
    }

    let mut pub_room = rm.pub_rooms.get_mut(&room_model).unwrap();
    //校验玩家是否在房间里
    let res = pub_room.get_mut_room_by_user_id(&user_id);
    if res.is_ok() {
        let str = format!(
            "this player already in the room,room_id:{},user_id:{}",
            res.unwrap().get_room_id(),
            user_id
        );
        let mut sr = S_ROOM::new();
        sr.is_succ = false;
        sr.err_mess = str;
        let bytes = Packet::build_packet_bytes(
            ClientCode::Room as u32,
            packet.get_user_id(),
            sr.write_to_bytes()?,
            true,
            true,
        );
        rm.sender.as_mut().unwrap().write(bytes)?;
        return Ok(());
    }
    //执行正常流程
    let member = Member::from(grs.take_pbp());
    let res = pub_room.quickly_start(member);
    //返回客户端
    let mut sr = S_ROOM::new();
    if res.is_err() {
        let str = res.err().unwrap().to_string();
        sr.is_succ = false;
        sr.err_mess = str;
        let bytes = Packet::build_packet_bytes(
            ClientCode::Room as u32,
            packet.get_user_id(),
            sr.write_to_bytes()?,
            true,
            true,
        );
        rm.sender.as_mut().unwrap().write(bytes)?;
        return Ok(());
    }
    let room = res.unwrap();
    sr.is_succ = true;
    sr.set_room(room.convert_to_pt());
    let bytes = Packet::build_packet_bytes(
        ClientCode::Room as u32,
        packet.get_user_id(),
        sr.write_to_bytes()?,
        true,
        true,
    );
    let res = rm.sender.as_mut().unwrap().write(bytes);
    if res.is_err() {
        let str = format!("{:?}", res.err().unwrap().to_string());
        error!("{:?}", str.as_str());
        anyhow::bail!("{:?}", str)
    }
    Ok(())
}

///准备
fn prepare_cancel(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    //校验玩家是否在房间
    // let res = rm.player_room.contains_key(&packet.get_user_id());
    // if !res {
    //     return;
    // }
    Ok(())
}

///开始
fn start(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
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
    Ok(())
}

///换队伍
fn change_team(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    // let user_id = &packet.get_user_id();
    // let room = check_player_in_room(user_id, rm);
    // if room.is_none() {
    //     return;
    // }
    // let room = room.unwrap();
    // room.change_team(&packet.get_user_id(), &(0 as u8));
    Ok(())
}

///T人
fn kick_member(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
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
    Ok(())
}

fn join_room(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut grj = G_R_JOIN_ROOM::new();
    grj.merge_from_bytes(packet.get_data());
    let room_id = grj.room_id;
    //校验玩家是否在房间内
    let res = rm.check_player(&user_id);
    let mut sr = S_ROOM::new();
    if res {
        let str = format!("this player already in the room!user_id:{}", user_id);
        sr.is_succ = false;
        sr.err_mess = str;
        let res = Packet::build_packet_bytes(
            ClientCode::Room as u32,
            user_id,
            sr.write_to_bytes()?,
            true,
            true,
        );
        rm.send(res);
        return Ok(());
    }

    //校验改房间是否存在
    let room = rm.friend_room.get_mut_room_by_room_id(&room_id);
    if room.is_err() {
        let str = room.err().unwrap().to_string();
        warn!("{:?}", str.as_str());
        sr.is_succ = false;
        sr.err_mess = str;
        let res = Packet::build_packet_bytes(
            ClientCode::Room as u32,
            user_id,
            sr.write_to_bytes()?,
            true,
            true,
        );
        rm.send(res);
        return Ok(());
    }

    //走正常逻辑
    let room = room.unwrap();
    let mut grjr = G_R_JOIN_ROOM::new();
    grjr.merge_from_bytes(packet.get_data());

    //校验玩家是否在房间里

    Ok(())
}
