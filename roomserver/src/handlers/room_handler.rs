use super::*;
use tools::cmd_code::ClientCode::RoomMemberNotice;

///创建房间
pub fn create_room(rm: &mut RoomMgr, mut packet: Packet) -> anyhow::Result<()> {
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
        match res {
            Ok(_) => {}
            Err(e) => {
                error!("{:?}", e.clone());
                anyhow::bail!("{:?}", e)
            }
        }
        return Ok(());
    }

    let owner = Member::from(grc.take_pbp());
    let mut room_id: u32 = 0;
    //创建房间
    if room_type == RoomType::get_custom() {
        room_id = rm
            .custom_room
            .create_room(owner, rm.sender.as_ref().unwrap().clone())?;
    } else if room_type == RoomType::get_season_pve() {
        error_str = Some("this function is not open yet!".to_owned());
    } else if room_type == RoomType::get_world_boss_pve() {
        error_str = Some("this function is not open yet!".to_owned());
    } else {
        let str = "could not create room,the room_type is invalid!".to_owned();
        warn!("{:?}", str);
    }
    if error_str.is_some() {
        warn!("{:?}", error_str.unwrap().as_str());
        return Ok(());
    }
    let res = tools::binary::combine_int_2_long(room_type as u32, room_id);
    rm.player_room.insert(packet.get_user_id(), res);
    Ok(())
}

///离开房间
pub fn leave_room(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let room_id = rm.get_room_id(&user_id);
    if room_id.is_none() {
        return Ok(());
    }

    //校验房间是否存在
    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        warn!("this player is not in the room!user_id:{}", user_id);
        return Ok(());
    }
    let room = room.unwrap();
    if packet.get_cmd() == RoomCode::LeaveRoom as u32 {
        let member = room.get_member_mut(&user_id);
        let member = member.unwrap();
        if member.state == MemberState::Ready as u8 {
            let mut slr = S_LEAVE_ROOM::new();
            let str = format!(
                "leave_room:this player is already ready!user_id:{}",
                user_id
            );
            slr.is_succ = false;
            slr.err_mess = str;
            let bytes = Packet::build_packet_bytes(
                ClientCode::LeaveRoom as u32,
                user_id,
                slr.write_to_bytes().unwrap(),
                true,
                true,
            );
            let res = rm.sender.as_mut().unwrap().write(bytes);
            match res {
                Ok(_) => {}
                Err(e) => error!("{:?}", e),
            }
            return Ok(());
        }
    }

    let room_id = room.get_room_id();
    let room_type = RoomType::from(room.get_room_type());
    let battle_type = room.setting.battle_type;
    match room_type {
        RoomType::Custom => {
            let res = rm.custom_room.leave_room(&room_id, &user_id);
            if res.is_err() {
                error!("{:?}", res.err().unwrap());
                return Ok(());
            }
            info!(
                "玩家离开自定义房间，卸载玩家房间数据!user_id:{},room_id:{}",
                user_id, room_id
            );
            rm.player_room.remove(&user_id);
        }
        RoomType::Match => {
            if room.is_empty() {
                let res = rm.match_rooms.leave(&battle_type, room_id, &user_id);
                if res.is_err() {
                    error!("{:?}", res.err().unwrap());
                    return Ok(());
                }
                let mut slr = S_LEAVE_ROOM::new();
                slr.set_is_succ(true);
                let bytes = Packet::build_packet_bytes(
                    ClientCode::LeaveRoom as u32,
                    user_id,
                    slr.write_to_bytes().unwrap(),
                    true,
                    true,
                );
                let res = rm.sender.as_mut().unwrap().write(bytes);
                if res.is_err() {
                    error!("{:?}", res.err().unwrap());
                }
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

///改变目标
pub fn change_target(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    Ok(())
}

///寻找房间并加入房间
pub fn search_room(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let mut grs = G_R_SEARCH_ROOM::new();
    let res = grs.merge_from_bytes(packet.get_data());
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
        return Ok(());
    }

    let battle_type = grs.battle_type as u8;
    let user_id = packet.get_user_id();
    //校验模式
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
    //返回错误信息
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
    Ok(())
}

///准备
pub fn prepare_cancel(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    let mut cpc = C_PREPARE_CANCEL::new();

    let res = cpc.merge_from_bytes(packet.get_data());
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
        return Ok(());
    }
    let room = rm.get_room_mut(&packet.get_user_id());

    if room.is_none() {
        let mut spc = S_PREPARE_CANCEL::new();
        spc.is_succ = false;
        spc.err_mess = "this player not in the room!".to_owned();
        let bytes = Packet::build_packet_bytes(
            ClientCode::PrepareCancel as u32,
            packet.get_user_id(),
            spc.write_to_bytes().unwrap(),
            true,
            true,
        );
        let res = rm.sender.as_mut().unwrap().write(bytes);
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
        }
        return Ok(());
    }

    let room = room.unwrap();
    room.prepare_cancel(&user_id, cpc.prepare);
    Ok(())
}

///开始
pub fn start(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
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
pub fn change_team(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = &packet.get_user_id();

    let mut cct = C_CHANGE_TEAM::new();
    let res = cct.merge_from_bytes(packet.get_data());
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
        return Ok(());
    }
    let team_id = cct.get_target_team_id();
    if team_id < TeamId::Min as u32 || team_id > TeamId::Max as u32 {
        let str = format!("target_team_id:{} is invaild!", team_id);
        warn!("{:?}", str.as_str());
        let mut sct = S_CHANGE_TEAM::new();
        sct.is_succ = false;
        sct.err_mess = str;
        let bytes = Packet::build_packet_bytes(
            ClientCode::ChangeTeam as u32,
            *user_id,
            sct.write_to_bytes().unwrap(),
            true,
            true,
        );
        let res = rm.sender.as_mut().unwrap().write(bytes);
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
        }
        return Ok(());
    }
    let room_id = rm.get_room_id(user_id);
    if room_id.is_none() {
        let str = format!("this player is not in the room!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        let mut sct = S_CHANGE_TEAM::new();
        sct.is_succ = false;
        sct.err_mess = str;
        let bytes = Packet::build_packet_bytes(
            ClientCode::ChangeTeam as u32,
            *user_id,
            sct.write_to_bytes().unwrap(),
            true,
            true,
        );
        let res = rm.sender.as_mut().unwrap().write(bytes);
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
        }
        return Ok(());
    }
    let room_id = room_id.unwrap();
    let room = rm.custom_room.rooms.get_mut(&room_id);
    if room.is_none() {
        let str = format!("this player is not in the room!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        let mut sct = S_CHANGE_TEAM::new();
        sct.is_succ = false;
        sct.err_mess = str;
        let bytes = Packet::build_packet_bytes(
            ClientCode::ChangeTeam as u32,
            *user_id,
            sct.write_to_bytes().unwrap(),
            true,
            true,
        );
        let res = rm.sender.as_mut().unwrap().write(bytes);
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
        }
        return Ok(());
    }

    let room = room.unwrap();
    room.change_team(user_id, &(team_id as u8));
    Ok(())
}

///T人
pub fn kick_member(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    let mut ckm = C_KICK_MEMBER::new();
    let res = ckm.merge_from_bytes(packet.get_data());
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
        return Ok(());
    }
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
    } else {
        rm.player_room.remove(&target_id);
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
        let str = format!("this player is not in the room,room_id:{}", user_id);
        srs.is_succ = false;
        srs.err_mess = str.clone();
        warn!("{:?}", str.as_str());
    }
    let room = room.unwrap();

    //校验房间是否存在这个玩家
    if !room.is_exist_member(&user_id) {
        let str = format!("this player is not in the room,room_id:{}", user_id);
        srs.is_succ = false;
        srs.err_mess = str.clone();
        warn!("{:?}", str.as_str());
    }

    //校验玩家是否是房主
    if room.get_owner_id() != user_id {
        srs.is_succ = false;
        let str = format!(
            "this player is not master:{},room_id:{}",
            user_id,
            room.get_room_id()
        );
        srs.is_succ = false;
        srs.err_mess = str.clone();
        warn!("{:?}", str.as_str());
    }

    //走正常逻辑
    if srs.is_succ {
        let mut rs = C_ROOM_SETTING::new();
        let res = rs.merge_from_bytes(packet.get_data());
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
        }
        let rs_pt = rs.take_setting();
        let rs = crate::entity::battle_model::RoomSetting::from(rs_pt);
        room.set_room_setting(rs);
    }

    //回给客户端
    let bytes = Packet::build_packet_bytes(
        ClientCode::RoomSetting as u32,
        user_id,
        srs.write_to_bytes().unwrap(),
        true,
        true,
    );
    let res = room.sender.write(bytes);
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
    }
    room.room_notice(&user_id);
    Ok(())
}

///加入房间
pub fn join_room(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut grj = G_R_JOIN_ROOM::new();
    let res = grj.merge_from_bytes(packet.get_data());
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
        return Ok(());
    }
    if grj.room_type == 0
        || grj.room_type > RoomType::get_world_boss_pve() as u32
        || grj.room_type == RoomType::get_match() as u32
    {
        warn!("room_type is error!");
        return Ok(());
    }

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
    let res = room.add_member(member);
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
        return Ok(());
    }
    let value = tools::binary::combine_int_2_long(grj.room_type, res.unwrap());
    rm.player_room.insert(user_id, value);
    Ok(())
}

///选择角色
pub fn choose_character(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);
    let mut scc = S_CHOOSE_CHARACTER::new();
    scc.is_succ = true;
    //校验玩家在不在房间
    if res.is_none() {
        scc.is_succ = false;
        let str = format!("this player is not in room!user_id:{}", user_id);
        scc.err_mess = str.clone();
        warn!("{:?}", str.as_str());
        let bytes = Packet::build_packet_bytes(
            ClientCode::ChooseCharacter as u32,
            user_id,
            scc.write_to_bytes().unwrap(),
            true,
            true,
        );
        let res = rm.sender.as_mut().unwrap().write(bytes);
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
        }
        return Ok(());
    }
    let room = res.unwrap();
    //校验房间状态
    if room.get_status() == RoomState::Started as u8 {
        scc.is_succ = false;
        let str = format!("this room already started!room_id:{}", room.get_room_id());
        scc.err_mess = str.clone();
        warn!("{:?}", str.as_str());
    }

    //走正常流程
    let mut ccc = C_CHOOSE_CHARACTER::new();
    let res = ccc.merge_from_bytes(packet.get_data());
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
    }

    let cter_c = ccc.take_cter();

    //校验玩家
    let member = room.get_member_mut(&user_id);
    if member.is_none() {
        scc.is_succ = false;
        let str = format!("this player is not in room!user_id:{}", user_id);
        scc.err_mess = str.clone();
        warn!("{:?}", str.as_str());
    }
    let member = member.unwrap();

    //校验玩家状态
    if member.state == MemberState::Ready as u8 {
        scc.is_succ = false;
        let str = format!("this player is already prepare!user_id:{}", user_id);
        scc.err_mess = str.clone();
        warn!("{:?}", str.as_str());
    }

    let cter = member.cters.get(&cter_c.get_temp_id());
    if cter_c.get_temp_id() > 0 && cter.is_none() {
        scc.is_succ = false;
        let str = format!(
            "this player do not have this character!user_id:{},cter_id:{}",
            user_id,
            cter_c.get_temp_id()
        );
        scc.err_mess = str.clone();
        warn!("{:?}", str.as_str());
    }
    let cter = cter.unwrap();
    for skill in cter_c.skills.iter() {
        if !cter.skills.contains(skill) {
            scc.is_succ = false;
            let str = format!(
                "this do not have this skill!user_id:{},cter_id:{},skill_id:{}",
                user_id,
                cter_c.get_temp_id(),
                *skill
            );
            scc.err_mess = str.clone();
            let bytes = Packet::build_packet_bytes(
                ClientCode::ChooseCharacter as u32,
                user_id,
                scc.write_to_bytes().unwrap(),
                true,
                true,
            );
            let res = room.sender.write(bytes);
            if res.is_err() {
                error!("{:?}", res.err().unwrap().to_string());
            }
            warn!("{:?}", str.as_str());
            return Ok(());
        }
    }
    //走正常逻辑
    if scc.is_succ {
        //校验角色技能
        let mut choice_cter = Charcter::default();
        choice_cter.clone_from(cter);
        member.chose_cter = choice_cter;
    }
    //返回客户端
    let bytes = Packet::build_packet_bytes(
        ClientCode::ChooseCharacter as u32,
        user_id,
        scc.write_to_bytes().unwrap(),
        true,
        true,
    );
    let res = room.sender.write(bytes);
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
    }
    //通知其他成员
    room.room_member_notice(RoomMemberNoticeType::UpdateMember as u8, &user_id);
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
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
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
        let mut se = S_EMOJI::new();
        se.is_succ = false;
        let str = format!("this emoji need unlock!emoji_id:{}", emoji_id);
        warn!("{:?}", str.as_str());
        se.err_mess = str;
        let res = Packet::build_packet_bytes(
            ClientCode::Emoji as u32,
            user_id,
            se.write_to_bytes().unwrap(),
            true,
            true,
        );
        room.sender.write(res)?;
        return Ok(());
    } else if emoji.condition == 0 && emoji.cter_id != member.chose_cter.temp_id {
        let mut se = S_EMOJI::new();
        se.is_succ = false;
        let str = format!(
            "this character can not send this emoji!cter_id:{},emoji_id:{}",
            member.chose_cter.temp_id, emoji_id
        );
        warn!("{:?}", str.as_str());
        se.err_mess = str;
        let res = Packet::build_packet_bytes(
            ClientCode::Emoji as u32,
            user_id,
            se.write_to_bytes().unwrap(),
            true,
            true,
        );
        room.sender.write(res)?;
        return Ok(());
    }
    //走正常逻辑
    room.emoji(user_id, emoji_id);
    Ok(())
}
