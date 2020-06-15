use super::*;
use crate::entity::battle_model::{
    BattleType, CustomRoom, MatchRoom, MatchRooms, RoomModel, RoomType,
};
use crate::entity::member::{BattleCharcter, Member, MemberState, UserType};
use crate::entity::room::{member_2_memberpt, Room, RoomState};
use crate::TEMPLATES;
use log::{error, info, warn};
use protobuf::Message;
use std::hash::Hash;
use tools::cmd_code::ClientCode;
use tools::cmd_code::RoomCode::RoomSetting;
use tools::protos::base::RoomPt;
use tools::protos::room::{
    C_CHANGE_TEAM, C_CHOICE_CHARACTER, C_JOIN_ROOM, C_KICK_MEMBER, C_PREPARE_CANCEL,
    C_ROOM_SETTING, S_ROOM, S_ROOM_MEMBER_NOTICE,
};
use tools::protos::server_protocol::{
    PlayerBattlePt, G_R_CREATE_ROOM, G_R_JOIN_ROOM, G_R_SEARCH_ROOM,
};
use tools::templates::character_temp::CharacterTemp;
use tools::templates::tile_map_temp::TileMapTempMgr;
use tools::util::packet::Packet;

//房间服管理器
pub struct RoomMgr {
    pub custom_room: CustomRoom,        //自定义房
    pub match_rooms: MatchRooms,        //公共房
    pub player_room: HashMap<u32, u64>, //玩家对应的房间，key:u32,value:采用一个u64存，通过位运算分出高低位,低32位是房间模式,告32位是房间id
    pub cmd_map: HashMap<u32, fn(&mut RoomMgr, Packet) -> anyhow::Result<()>, RandomState>, //命令管理 key:cmd,value:函数指针
    pub sender: Option<TcpSender>, //tcp channel的发送方
}

impl RoomMgr {
    pub fn new() -> RoomMgr {
        let cmd_map: HashMap<u32, fn(&mut RoomMgr, Packet) -> anyhow::Result<()>, RandomState> =
            HashMap::new();
        let custom_room = CustomRoom::default();
        let match_rooms = MatchRooms::default();
        let player_room: HashMap<u32, u64> = HashMap::new();
        let mut rm = RoomMgr {
            custom_room,
            match_rooms,
            player_room,
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

    pub fn get_room_id(&self, user_id: &u32) -> Option<u32> {
        let res = self.player_room.get(user_id);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let (_, room_id) = tools::binary::separate_long_2_int(*res);
        return Some(room_id);
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

    pub fn send(&mut self, bytes: Vec<u8>) {
        if self.sender.is_none() {
            error!("room_mgr'sender is None!");
            return;
        }
        self.sender.as_mut().unwrap().write(bytes);
    }

    pub fn get_room_mut(&mut self, user_id: &u32) -> Option<&mut Room> {
        let res = self.player_room.get(user_id);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let (model, room_id) = tools::binary::separate_long_2_int(*res);

        if model == RoomType::into_u32(RoomType::Custom) {
            return self.custom_room.get_room_mut(&room_id);
        } else if model == RoomType::into_u32(RoomType::Match) {
            return self.match_rooms.get_room_mut(&room_id);
        } else if model == RoomType::into_u32(RoomType::SeasonPve) {
            return None;
        }
        None
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
        self.cmd_map
            .insert(RoomCode::RoomSetting as u32, room_setting);
        self.cmd_map
            .insert(RoomCode::ChoiceCharacter as u32, chioce_character);
    }
}

///创建房间
fn create_room(rm: &mut RoomMgr, mut packet: Packet) -> anyhow::Result<()> {
    //解析protobuf
    let mut grc = G_R_CREATE_ROOM::new();
    grc.merge_from_bytes(packet.get_data())?;

    let room_type = grc.get_room_type() as u8;
    let user_id = packet.get_user_id();

    let mut error_str: Option<String> = None;

    //校验玩家是否在房间内
    if room_type == RoomType::get_custom() {
        //校验这个用户在不在房间内
        let res = rm.get_room_id(&packet.get_user_id());
        if res.is_some() {
            error_str = Some(format!(
                "this user already in the custom room,can not create room! user_id:{},room_id:{}",
                user_id,
                res.unwrap()
            ));
            warn!("{:?}", error_str.as_ref().unwrap().as_str());
        }
    } else if room_type == RoomType::get_season_pve() {
        error_str = Some("this function is not open yet!".to_owned());
    } else if room_type == RoomType::get_world_boss_pve() {
        error_str = Some("this function is not open yet!".to_owned());
    } else {
        let str = "could not create room,the room_type is invalid!".to_owned();
        warn!("{:?}", str);
        return Ok(());
    }

    //如果有问题，错误信息返回给客户端
    if error_str.is_some() {
        let str = error_str.unwrap();
        let mut sr = S_ROOM::new();
        sr.err_mess = str;
        sr.is_succ = false;
        let res = sr.write_to_bytes().unwrap();
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
        return Ok(());
    }

    let owner = Member::from(grc.take_pbp());
    let mut room: Option<&mut Room> = None;
    //创建房间
    if room_type == RoomType::get_custom() {
        let room_id = rm
            .custom_room
            .create_room(owner, rm.sender.as_ref().unwrap().clone())?;

        room = Some(rm.custom_room.rooms.get_mut(&room_id).unwrap());
    } else if room_type == RoomType::get_season_pve() {
        error_str = Some("this function is not open yet!".to_owned());
    } else if room_type == RoomType::get_world_boss_pve() {
        error_str = Some("this function is not open yet!".to_owned());
    } else {
        let str = "could not create room,the room_type is invalid!".to_owned();
        warn!("{:?}", str);
        return Ok(());
    }

    if room.is_none() {
        return Ok(());
    }
    let room = room.unwrap();
    let res = tools::binary::combine_int_2_long(room_type as u32, room.get_room_id());
    rm.player_room.insert(packet.get_user_id(), res);
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
    let room_id = rm.get_room_id(&user_id);
    if room_id.is_none() {
        return Ok(());
    }
    let room_id = room_id.unwrap();
    //处理好友房
    let res = rm.custom_room.leave_room(&room_id, &user_id);
    match res {
        Ok(_) => {
            info!(
                "卸载玩家好友房数据！user_id:{},room_id:{}",
                user_id, room_id
            );
            rm.player_room.remove(&user_id);
            return Ok(());
        }
        Err(_) => {}
    }
    //处理随机房
    let res = rm.match_rooms.leave(room_id, &user_id);
    if let Some(_) = res {
        info!(
            "卸载玩家公共match房数据！user_id:{},room_id:{}",
            user_id, room_id
        );
        rm.player_room.remove(&user_id);
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

    let battle_type = grs.battle_type as u8;
    let user_id = packet.get_user_id();
    ///校验模式
    if battle_type < BattleType::OneVOneVOneVOne as u8 || battle_type > BattleType::OneVOne as u8 {
        let s = format!("this model is not exist!model_type:{}", battle_type);
        anyhow::bail!(s)
    }

    let mut sr = S_ROOM::new();
    //校验玩家是否已经在房间里
    if rm.check_player(&user_id) {
        let str = format!("this player already in the room!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
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
    let match_room = rm.match_rooms.get_match_room_mut(&battle_type);
    let member = Member::from(grs.take_pbp());

    let res = match_room.quickly_start(member, rm.sender.as_ref().unwrap().clone());
    //返回客户端
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
    };
    let room_id = res.unwrap();
    let value = tools::binary::combine_int_2_long(RoomType::Match as u32, room_id);
    rm.player_room.insert(packet.get_user_id(), value);
    //返回客户端
    let room = rm.get_room_mut(&user_id).unwrap();
    sr.is_succ = true;
    sr.set_room(room.convert_to_pt());
    let bytes = Packet::build_packet_bytes(
        ClientCode::Room as u32,
        packet.get_user_id(),
        sr.write_to_bytes()?,
        true,
        true,
    );

    //返回客户端
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
    let user_id = packet.get_user_id();

    let mut cpc = C_PREPARE_CANCEL::new();

    cpc.merge_from_bytes(packet.get_data());
    let room = rm.custom_room.get_room_mut(&packet.get_user_id());

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
    let user_id = &packet.get_user_id();
    let room_id = rm.get_room_id(user_id);
    if room_id.is_none() {
        return Ok(());
    }
    let room_id = room_id.unwrap();
    let room = rm.custom_room.rooms.get_mut(&room_id);
    if room.is_none() {
        error!("there is no friend room for room_id:{}", room_id);
        return Ok(());
    }

    let mut cct = C_CHANGE_TEAM::new();
    cct.merge_from_bytes(packet.get_data());
    let team_id = cct.get_target_team_id();

    let room = room.unwrap();
    room.change_team(user_id, &(team_id as u8));

    let room_pt = room.convert_to_pt();

    let mut sr = S_ROOM::new();
    sr.is_succ = true;
    sr.set_room(room_pt);
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

///T人
fn kick_member(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    let mut ckm = C_KICK_MEMBER::new();
    ckm.merge_from_bytes(packet.get_data());
    let target_id = ckm.target_id;

    //校验房间
    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        return Ok(());
    }

    //校验操作人是不是房主
    let room = room.unwrap();
    if room.get_owner_id() != user_id {
        return Ok(());
    }

    //校验房间是否存在target_id这个成员
    if !room.is_exist_member(&target_id) {
        return Ok(());
    }

    let res = room.kick_member(&user_id, &target_id);
    if res.is_err() {
        warn!("{:?}", res.err().unwrap().to_string());
        return Ok(());
    }
    //回给客户端

    Ok(())
}

///房间设置
fn room_setting(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        return Ok(());
    }
    let room = room.unwrap();

    //校验房间是否存在这个玩家
    if !room.is_exist_member(&user_id) {
        return Ok(());
    }

    //校验玩家是否是房主
    if room.get_owner_id() != user_id {
        return Ok(());
    }

    let mut rs = C_ROOM_SETTING::new();
    rs.merge_from_bytes(packet.get_data());
    let rs_pt = rs.take_setting();
    let rs = crate::entity::battle_model::RoomSetting::from(rs_pt);
    room.set_room_setting(rs);
    //回给客户端
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
    let room = rm.custom_room.get_mut_room_by_room_id(&room_id);
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

    //校验房间人数
    if room.members.len() >= 4 {
        let str = format!("this room already have max player num!,room_id:{}", room_id);
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

    // 校验玩家是否在房间里
    let res = room.is_exist_member(&packet.get_user_id());
    if res {
        let str = format!(
            "this player already in the room!user_id:{},room_id:{}",
            packet.get_user_id(),
            room_id
        );
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
    let member = Member::from(grj.take_pbp());
    //将玩家加入到房间
    room.add_member(member);
    //返回客户端消息
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

///选择角色
fn chioce_character(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);
    //校验玩家在不在房间
    if res.is_none() {
        return Ok(());
    }

    let room = res.unwrap();

    //校验房间状态
    if room.get_status() == RoomState::Started as u8 {
        return Ok(());
    }

    //走正常流程
    let mut ccc = C_CHOICE_CHARACTER::new();
    ccc.merge_from_bytes(packet.get_data());

    let cter_c = ccc.take_cter();

    //校验角色
    let member = room.get_member_mut_by_user_id(&user_id);
    if member.is_none() {
        return Ok(());
    }
    let member = member.unwrap();
    let cter = member.cters.get(&cter_c.get_temp_id());
    if cter.is_none() {
        return Ok(());
    }
    let cter = cter.unwrap();
    for skill in cter_c.skills.iter() {
        if !cter.skills.contains(skill) {
            return Ok(());
        }
    }

    //校验角色技能
    member.battle_cter = BattleCharcter::default();
    member.battle_cter.skills = cter_c.skills;
    member.battle_cter.temp_id = cter_c.temp_id;

    //同志客户端
    Ok(())
}
