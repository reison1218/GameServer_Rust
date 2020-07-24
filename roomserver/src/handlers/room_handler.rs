use super::*;
use crate::entity::character::Character;
use crate::entity::room::{MemberLeaveNoticeType, RoomState, MEMBER_MAX};
use crate::error_return::err_back;
use tools::protos::room::{
    C_CHOOSE_INDEX, C_CHOOSE_SKILL, C_CHOOSE_TURN_ORDER, S_CHOOSE_CHARACTER,
    S_CHOOSE_CHARACTER_NOTICE, S_CHOOSE_SKILL, S_START,
};

///创建房间
pub fn create_room(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    //解析gameserver过来的protobuf
    let mut grc = G_R_CREATE_ROOM::new();
    grc.merge_from_bytes(packet.get_data())?;

    let room_type = grc.get_room_type() as u8;
    let user_id = packet.get_user_id();

    //校验玩家是否在房间内
    if room_type == RoomType::get_custom() {
        //校验这个用户在不在房间内
        let res = rm.get_room_id(&packet.get_user_id());
        if let Some(room_id) = res {
            let str = format!(
                "this user already in the custom room,can not create room! user_id:{},room_id:{}",
                user_id, room_id
            );
            warn!("{:?}", str.as_str());
            err_back(ClientCode::Room, user_id, str, rm.get_sender_mut());
            return Ok(());
        }
    } else if room_type == RoomType::get_season_pve() {
        let str = "this function is not open yet!".to_owned();
        warn!("{:?}", str.as_str());
        err_back(ClientCode::Room, user_id, str, rm.get_sender_mut());
        return Ok(());
    } else if room_type == RoomType::get_world_boss_pve() {
        let str = "this function is not open yet!".to_owned();
        warn!("{:?}", str.as_str());
        err_back(ClientCode::Room, user_id, str, rm.get_sender_mut());
        return Ok(());
    } else {
        let str = "could not create room,the room_type is invalid!".to_owned();
        warn!("{:?}", str.as_str());
        err_back(ClientCode::Room, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    let owner = Member::from(grc.take_pbp());
    let mut room_id: u32 = 0;
    let room_type = RoomType::from(room_type);
    //创建房间
    match room_type {
        RoomType::Custom => {
            room_id = rm.custom_room.create_room(
                BattleType::None as u8,
                owner,
                rm.get_sender_clone(),
                rm.get_task_sender_clone(),
            )?;
        }
        _ => {}
    }

    let res = tools::binary::combine_int_2_long(room_type as u32, room_id);
    rm.player_room.insert(packet.get_user_id(), res);
    Ok(())
}

///离开房间
pub fn leave_room(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    //校验房间是否存在
    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        return Ok(());
    }
    let room = room.unwrap();

    //如果不再等待阶段，不允许主动推出房间
    if room.get_state() != &RoomState::Await && packet.get_cmd() == RoomCode::LeaveRoom as u32 {
        warn!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        return Ok(());
    }
    let room_id = room.get_room_id();
    if packet.get_cmd() == RoomCode::LeaveRoom as u32 {
        let member = room.get_member_mut(&user_id);
        let member = member.unwrap();
        if member.state == MemberState::Ready as u8 {
            let str = format!(
                "leave_room:this player is already ready!user_id:{}",
                user_id
            );
            warn!("{:?}", str.as_str());
            err_back(ClientCode::LeaveRoom, user_id, str, room.get_sender_mut());
            return Ok(());
        }
    }
    let room_type = RoomType::from(room.get_room_type());
    let battle_type = room.setting.battle_type;
    match room_type {
        RoomType::Custom => {
            let res =
                rm.custom_room
                    .leave_room(MemberLeaveNoticeType::Leave as u8, &room_id, &user_id);
            if let Err(e) = res {
                error!("{:?}", e);
                return Ok(());
            }
            info!(
                "玩家离开自定义房间，卸载玩家房间数据!user_id:{},room_id:{}",
                user_id, room_id
            );
            rm.player_room.remove(&user_id);
        }
        RoomType::Match => {
            if !room.is_empty() {
                let res = rm.match_rooms.leave(&battle_type, room_id, &user_id);
                if let Err(e) = res {
                    error!("{:?}", e);
                    return Ok(());
                }
                let mut slr = S_LEAVE_ROOM::new();
                slr.set_is_succ(true);
                rm.send_2_client(
                    ClientCode::LeaveRoom,
                    user_id,
                    slr.write_to_bytes().unwrap(),
                );
                info!(
                    "玩家离开匹配房间，卸载玩家房间数据!user_id:{},room_id:{}",
                    user_id, room_id
                );
            }
            rm.player_room.remove(&user_id);
        }
        _ => {}
    }
    Ok(())
}

///寻找房间并加入房间
pub fn search_room(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let mut grs = G_R_SEARCH_ROOM::new();
    let res = grs.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }

    let battle_type = grs.battle_type as u8;
    let user_id = packet.get_user_id();
    //校验模式
    if battle_type < BattleType::OneVOneVOneVOne as u8 || battle_type > BattleType::OneVOne as u8 {
        let s = format!(
            "search_room:this model is not exist!model_type:{}",
            battle_type
        );
        err_back(ClientCode::Room, user_id, s, rm.get_sender_mut());
        return Ok(());
    }

    //校验玩家是否已经在房间里
    if rm.check_player(&user_id) {
        let str = format!(
            "search_room:this player already in the room!user_id:{}",
            user_id
        );
        warn!("{:?}", str.as_str());
        err_back(ClientCode::Room, user_id, str, rm.get_sender_mut());
        return Ok(());
    }
    //执行正常流程
    let sender = rm.get_sender_clone();
    let match_room = rm.match_rooms.get_match_room_mut(&battle_type);
    let member = Member::from(grs.take_pbp());

    let res = match_room.quickly_start(member, sender, rm.task_sender.clone().unwrap());
    //返回错误信息
    if let Err(e) = res {
        let str = e.to_string();
        err_back(ClientCode::Room, user_id, str, rm.get_sender_mut());
        return Ok(());
    };
    let room_id = res.unwrap();
    let value = tools::binary::combine_int_2_long(RoomType::Match as u32, room_id);
    rm.player_room.insert(user_id, value);
    Ok(())
}

///准备
pub fn prepare_cancel(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut cpc = C_PREPARE_CANCEL::new();
    let res = cpc.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let prepare = cpc.prepare;
    let room = rm.get_room_mut(&packet.get_user_id());
    //校验玩家房间
    if let None = room {
        let str = format!(
            "prepare_cancel:this player not in the room!user_id:{}",
            user_id
        );
        err_back(ClientCode::PrepareCancel, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    let room = room.unwrap();
    //校验房间是否已经开始游戏
    if room.get_state() != &RoomState::Await {
        let str = format!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        anyhow::bail!(str)
    }
    //校验玩家是否选了角色
    let member = room.members.get(&user_id).unwrap();
    let cter_id = member.chose_cter.cter_id;
    if cter_id == 0 {
        let str = format!(
            "prepare_cancel: this player has not choose character yet!user_id:{}",
            user_id
        );
        err_back(
            ClientCode::PrepareCancel,
            user_id,
            str,
            room.get_sender_mut(),
        );
        return Ok(());
    }

    let cter = crate::TEMPLATES
        .get_character_ref()
        .temps
        .get(&cter_id)
        .unwrap();

    //校验玩家是否选了技能
    if prepare && member.chose_cter.skills.len() < cter.usable_skill_count as usize {
        let str = format!(
            "prepare_cancel: this player has not choose character'skill yet!user_id:{}",
            user_id
        );
        err_back(
            ClientCode::PrepareCancel,
            user_id,
            str,
            room.get_sender_mut(),
        );
        return Ok(());
    }
    room.prepare_cancel(&user_id, prepare);
    Ok(())
}

///开始游戏
pub fn start(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    //校验房间
    let room = rm.get_room_mut(&user_id);
    if let None = room {
        let str = format!("this player is not in the room!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        err_back(ClientCode::Start, user_id, str, rm.get_sender_mut());
        return Ok(());
    }
    let room = room.unwrap();

    //校验房间是否已经开始游戏
    if room.is_started() {
        let str = format!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        anyhow::bail!(str)
    }

    //校验准备状态
    if !room.check_ready() {
        let str = format!("there is player not ready,can not start game!");
        warn!("{:?}", str.as_str());
        err_back(ClientCode::Start, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    //执行开始逻辑
    room.start();

    let mut ss = S_START::new();
    ss.is_succ = true;
    room.send_2_client(ClientCode::Start, user_id, ss.write_to_bytes().unwrap());
    Ok(())
}

///换队伍
pub fn change_team(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = &packet.get_user_id();

    let mut cct = C_CHANGE_TEAM::new();
    let res = cct.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let team_id = cct.get_target_team_id();
    if team_id < TeamId::Min as u32 || team_id > TeamId::Max as u32 {
        let str = format!("target_team_id:{} is invaild!", team_id);
        warn!("{:?}", str.as_str());
        return Ok(());
    }
    let room_id = rm.get_room_id(user_id);
    if let None = room_id {
        let str = format!("this player is not in the room!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        return Ok(());
    }
    let room_id = room_id.unwrap();
    let room = rm.custom_room.rooms.get_mut(&room_id);
    if let None = room {
        let str = format!("this player is not in the room!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        return Ok(());
    }

    let room = room.unwrap();

    //校验房间是否已经开始游戏
    if room.is_started() {
        let str = format!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        anyhow::bail!(str)
    }
    room.change_team(user_id, &(team_id as u8));
    Ok(())
}

///T人
pub fn kick_member(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    let mut ckm = C_KICK_MEMBER::new();
    let res = ckm.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let target_id = ckm.target_id;
    //校验房间
    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        let str = format!(
            "kick_member:this player is not in the room!user_id:{}",
            user_id
        );
        err_back(ClientCode::KickMember, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    //校验操作人是不是房主
    let room = room.unwrap();

    //校验房间是否已经开始游戏
    if room.is_started() {
        let str = format!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        anyhow::bail!(str)
    }

    if room.get_room_type() != RoomType::get_custom() {
        let str = format!(
            "kick_member:this room is not custom room,can not kick member!room_id:{}",
            room.get_room_id()
        );
        err_back(ClientCode::KickMember, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    if room.get_owner_id() != user_id {
        let str = format!("kick_member:this player is not host!user_id:{}", user_id);
        err_back(ClientCode::KickMember, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    //校验房间是否存在target_id这个成员
    if !room.is_exist_member(&target_id) {
        let str = format!(
            "kick_member:this target player is not in the room!target_user_id:{}",
            target_id
        );
        err_back(ClientCode::KickMember, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    let res = room.kick_member(&user_id, &target_id);
    match res {
        Ok(_) => {
            rm.player_room.remove(&target_id);
        }
        Err(e) => {
            warn!("{:?}", e);
            return Ok(());
        }
    }
    Ok(())
}

///房间设置
pub fn room_setting(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let room = rm.get_room_mut(&user_id);
    let mut srs = S_ROOM_SETTING::new();
    srs.is_succ = true;
    if room.is_none() {
        let str = format!(
            "room_setting:this player is not in the room,room_id:{}",
            user_id
        );
        warn!("{:?}", str.as_str());
        err_back(ClientCode::RoomSetting, user_id, str, rm.get_sender_mut());
        return Ok(());
    }
    let room = room.unwrap();

    //校验房间是否已经开始游戏
    if room.is_started() {
        let str = format!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        anyhow::bail!(str)
    }

    //校验房间是否存在这个玩家
    if !room.is_exist_member(&user_id) {
        let str = format!(
            "room_setting:this player is not in the room,room_id:{}",
            user_id
        );
        warn!("{:?}", str.as_str());
        err_back(ClientCode::RoomSetting, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    //校验玩家是否是房主
    if room.get_owner_id() != user_id {
        let str = format!(
            "this player is not master:{},room_id:{}",
            user_id,
            room.get_room_id()
        );
        warn!("{:?}", str.as_str());
        err_back(ClientCode::RoomSetting, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    //走正常逻辑
    if srs.is_succ {
        let mut rs = C_ROOM_SETTING::new();
        let res = rs.merge_from_bytes(packet.get_data());
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
        }
        let rs_pt = rs.take_setting();
        let rs = crate::entity::room_model::RoomSetting::from(rs_pt);
        room.set_room_setting(rs);
    }

    //回给客户端
    room.send_2_client(
        ClientCode::RoomSetting,
        user_id,
        srs.write_to_bytes().unwrap(),
    );
    room.room_notice();
    Ok(())
}

///加入房间
pub fn join_room(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut grj = G_R_JOIN_ROOM::new();
    let res = grj.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }

    let room_id = grj.room_id;
    //校验玩家是否在房间内
    let res = rm.check_player(&user_id);
    if res {
        let str = format!("this player already in the room!user_id:{}", user_id);
        err_back(ClientCode::Room, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    //校验改房间是否存在
    let room = rm.custom_room.get_mut_room_by_room_id(&room_id);
    if let Err(e) = room {
        let str = e.to_string();
        warn!("{:?}", str.as_str());
        err_back(ClientCode::Room, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    let room = room.unwrap();

    //校验房间是否已经开始游戏
    if room.is_started() {
        let str = format!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        anyhow::bail!(str)
    }

    let room_type = room.get_room_type();
    //校验房间类型
    if room_type > RoomType::get_world_boss_pve() || room_type == RoomType::get_match() {
        warn!(
            "this room can not join in!room_id:{},room_type:{}!",
            room.get_room_id(),
            room_type,
        );
        return Ok(());
    }

    //校验房间人数
    if room.members.len() >= MEMBER_MAX as usize {
        let str = format!("this room already have max player num!,room_id:{}", room_id);
        warn!("{:?}", str.as_str());
        err_back(ClientCode::Room, user_id, str, rm.get_sender_mut());
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
        err_back(ClientCode::Room, user_id, str, rm.get_sender_mut());
        return Ok(());
    }
    let member = Member::from(grj.take_pbp());
    //将玩家加入到房间
    let res = room.add_member(member);
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let room_id = res.unwrap();
    let value = tools::binary::combine_int_2_long(room.get_room_type() as u32, room_id);
    rm.player_room.insert(user_id, value);
    Ok(())
}

///选择角色
pub fn choose_character(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);

    //校验玩家在不在房间
    if res.is_none() {
        let str = format!("this player is not in room!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        err_back(
            ClientCode::ChoiceCharacter,
            user_id,
            str,
            rm.get_sender_mut(),
        );
        return Ok(());
    }
    let room = res.unwrap();
    //校验房间状态
    if room.is_started() {
        let str = format!("this room already started!room_id:{}", room.get_room_id());
        warn!("{:?}", str.as_str());
        err_back(
            ClientCode::ChoiceCharacter,
            user_id,
            str,
            room.get_sender_mut(),
        );
        return Ok(());
    }

    //解析protobuf
    let mut ccc = C_CHOOSE_CHARACTER::new();
    let res = ccc.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let cter_id = ccc.cter_id;

    //校验角色
    let res = room.check_character(cter_id);
    if let Err(e) = res {
        let str = e.to_string();
        warn!("{:?}", str.as_str());
        err_back(
            ClientCode::ChoiceCharacter,
            user_id,
            str,
            room.get_sender_mut(),
        );
        return Ok(());
    }

    let member = room.get_member_mut(&user_id).unwrap();
    //校验玩家状态
    if member.state == MemberState::Ready as u8 {
        let str = format!("this player is already prepare!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        err_back(
            ClientCode::ChoiceCharacter,
            user_id,
            str,
            rm.get_sender_mut(),
        );
        return Ok(());
    }

    let cter = member.cters.get(&cter_id);
    //校验角色
    if cter_id > 0 && cter.is_none() {
        let str = format!(
            "this player do not have this character!user_id:{},cter_id:{}",
            user_id, cter_id
        );
        warn!("{:?}", str.as_str());
        err_back(
            ClientCode::ChoiceCharacter,
            user_id,
            str,
            rm.get_sender_mut(),
        );
        return Ok(());
    }
    if cter.is_some() {
        let cter = cter.unwrap();
        member.chose_cter = cter.clone();
    } else if cter_id == 0 {
        let choice_cter = Character::default();
        member.chose_cter = choice_cter;
    }

    let grade = member.chose_cter.grade;

    //走正常逻辑
    let mut scc = S_CHOOSE_CHARACTER::new();
    scc.is_succ = true;

    //返回客户端
    room.send_2_client(
        ClientCode::ChoiceCharacter,
        user_id,
        scc.write_to_bytes().unwrap(),
    );
    //通知其他成员
    let mut sccn = S_CHOOSE_CHARACTER_NOTICE::new();
    sccn.user_id = user_id;
    sccn.cter_id = cter_id;
    sccn.cter_grade = grade;
    let bytes = sccn.write_to_bytes().unwrap();
    let members = room.members.clone();
    for member_id in members.keys() {
        let mess = Packet::build_packet_bytes(
            ClientCode::ChoiceCharacterNotice as u32,
            *member_id,
            bytes.clone(),
            true,
            true,
        );
        room.get_sender_mut().write(mess);
    }
    Ok(())
}

///选择技能
pub fn choice_skills(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut ccs = C_CHOOSE_SKILL::new();
    let res = ccs.merge_from_bytes(packet.get_data());
    if res.is_err() {
        error!("{:?}", res.err().unwrap());
        return Ok(());
    }
    let skills = ccs.get_skills();

    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        let str = format!("this player is not in room!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        err_back(ClientCode::ChoiceSkill, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    let room = room.unwrap();
    let member = room.get_member_mut(&user_id).unwrap();
    if member.chose_cter.cter_id == 0 {
        let str = format!(
            "this player not choice cter yet!can not choice skill of cter!user_id:{}",
            user_id
        );
        warn!("{:?}", str.as_str());
        err_back(ClientCode::ChoiceSkill, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    let cter_id = member.chose_cter.cter_id;

    let cter = member.cters.get(&cter_id).unwrap();

    let cter_temp = crate::TEMPLATES
        .get_character_ref()
        .get_temp_ref(&cter_id)
        .unwrap();
    //校验技能数量
    if skills.len() > cter_temp.usable_skill_count as usize {
        let str = format!("this cter's skill count is error! cter_id:{}", cter_id);
        warn!("{:?}", str.as_str());
        err_back(ClientCode::ChoiceSkill, user_id, str, rm.get_sender_mut());
        return Ok(());
    }
    //校验技能有效性
    for skill in skills.iter() {
        if !cter.skills.contains(skill) {
            let str = format!(
                "this cter do not have this skill!user_id:{},cter_id:{},skill_id:{}",
                user_id, cter_id, *skill
            );
            warn!("{:?}", str.as_str());
            err_back(ClientCode::ChoiceSkill, user_id, str, rm.get_sender_mut());
            return Ok(());
        }
    }

    //校验技能合法性
    for group in cter_temp.skills.iter() {
        let mut count = 0;
        for skill in skills {
            if !group.group.contains(skill) {
                continue;
            }
            count += 1;
            if count >= 2 {
                let str = format!("the skill group is error!user_id:{}", user_id);
                warn!("{:?}", str.as_str());
                err_back(ClientCode::ChoiceSkill, user_id, str, rm.get_sender_mut());
                return Ok(());
            }
        }
    }

    //走正常逻辑
    member.chose_cter.skills = skills.to_vec();
    let mut scs = S_CHOOSE_SKILL::new();
    scs.is_succ = true;
    scs.skills = skills.to_vec();
    room.send_2_client(
        ClientCode::ChoiceSkill,
        user_id,
        scs.write_to_bytes().unwrap(),
    );
    Ok(())
}

///发送表情
pub fn emoji(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);
    if res.is_none() {
        error!("this player is not in the room!user_id:{}", user_id);
        return Ok(());
    }
    let room = res.unwrap();
    let member = room.get_member_mut(&user_id);
    if member.is_none() {
        error!("this player is not in the room!user_id:{}", user_id);
        return Ok(());
    }
    let member = member.unwrap();
    if member.state != MemberState::Ready as u8 {
        error!(
            "this player is not ready,can not send emoji!user_id:{}",
            user_id
        );
        return Ok(());
    }

    let mut ce = C_EMOJI::new();
    let res = ce.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let emoji_id = ce.emoji_id;
    let res: Option<&EmojiTemp> = crate::TEMPLATES.get_emoji_ref().temps.get(&emoji_id);
    if res.is_none() {
        error!("there is no temp for emoji_id:{}", emoji_id);
        return Ok(());
    }
    //校验表情是否需要解锁和角色表情
    let emoji = res.unwrap();
    if emoji.condition != 0 {
        let str = format!("this emoji need unlock!emoji_id:{}", emoji_id);
        warn!("{:?}", str.as_str());
        err_back(ClientCode::Emoji, user_id, str, rm.get_sender_mut());
        return Ok(());
    } else if emoji.condition == 0
        && emoji.cter_id > 0
        && emoji.cter_id != member.chose_cter.cter_id
    {
        let str = format!(
            "this character can not send this emoji!cter_id:{},emoji_id:{}",
            member.chose_cter.cter_id, emoji_id
        );
        warn!("{:?}", str.as_str());
        err_back(ClientCode::Emoji, user_id, str, rm.get_sender_mut());
        return Ok(());
    }
    //走正常逻辑
    room.emoji(user_id, emoji_id);
    Ok(())
}

///选择初始占位
pub fn choice_index(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut ccl = C_CHOOSE_INDEX::new();
    ccl.merge_from_bytes(packet.get_data())?;
    let index = ccl.index;

    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        let str = format!("this player is not in the room!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        err_back(ClientCode::ChoiceIndex, user_id, str, rm.get_sender_mut());
        return Ok(());
    }
    let room = room.unwrap();

    //校验参数
    if !room.battle_data.check_choice_index(index as usize) {
        let str = format!("the index is error!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        err_back(ClientCode::ChoiceIndex, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    //校验是否轮到他了
    if !room.is_can_choice_index_now(user_id) {
        let str =
            format!(
            "this player is not the next choice index player!user_id:{},index:{},choice_order:{:?}",
            user_id,room.get_next_choice_index(),room.battle_data.choice_orders
        );
        warn!("{:?}", str.as_str());
        err_back(ClientCode::ChoiceIndex, user_id, str, rm.get_sender_mut());
        return Ok(());
    }

    //校验他选过没有
    let member = room.get_battle_cter_ref(&user_id).unwrap();
    if member.cell_index != 0 {
        let str = format!("this player is already choice index!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        err_back(ClientCode::ChoiceIndex, user_id, str, rm.get_sender_mut());
        return Ok(());
    }
    room.choice_index(user_id, index);
    Ok(())
}

///选择回合顺序
pub fn choice_turn(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut ccl = C_CHOOSE_TURN_ORDER::new();
    ccl.merge_from_bytes(packet.get_data())?;
    let order = ccl.order;

    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        let str = format!("this player is not in the room!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        err_back(
            ClientCode::ChoiceRoundOrder,
            user_id,
            str,
            rm.get_sender_mut(),
        );
        return Ok(());
    }
    let room = room.unwrap();

    //校验参数
    if order > (MEMBER_MAX - 1) as u32 {
        let str = format!("the order's value is error!!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        err_back(
            ClientCode::ChoiceRoundOrder,
            user_id,
            str,
            rm.get_sender_mut(),
        );
        return Ok(());
    }

    //判断能不能选
    if !room.is_can_choice_turn_now(user_id) {
        let str = format!(
            "this player is not the next choice turn player!user_id:{},order:{:?}",
            user_id, room.battle_data.choice_orders
        );
        warn!("{:?}", str.as_str());
        err_back(
            ClientCode::ChoiceRoundOrder,
            user_id,
            str,
            rm.get_sender_mut(),
        );
        return Ok(());
    }

    //校验他选过没有
    if room.turn_order_contains(&user_id) {
        let str = format!(
            "this player is already choice round order!user_id:{}",
            user_id
        );
        warn!("{:?}", str.as_str());
        err_back(
            ClientCode::ChoiceRoundOrder,
            user_id,
            str,
            rm.get_sender_mut(),
        );
        return Ok(());
    }
    room.choice_turn(user_id, order as usize, true);
    Ok(())
}

///跳过选择回合顺序
pub fn skip_choice_turn(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        let str = format!("this player is not in the room!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        return Ok(());
    }
    let room = room.unwrap();
    //判断能不能选
    if !room.is_can_choice_turn_now(user_id) {
        let str = format!(
            "this player is not the next choice turn player!user_id:{}",
            user_id
        );
        warn!("{:?}", str.as_str());
        return Ok(());
    }
    room.skip_choice_turn(user_id);
    Ok(())
}
