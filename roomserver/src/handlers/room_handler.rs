use crate::mgr::room_mgr::RoomMgr;
use crate::mgr::RankInfo;
use crate::room::character::Character;
use crate::room::member::MemberState;
use crate::room::member::{Member, PunishMatch};
use crate::room::room::{MemberLeaveNoticeType, Room, RoomSettingType, RoomState, MEMBER_MAX};
use crate::room::room_model::{RoomModel, RoomType, TeamId};
use crate::task_timer::build_match_room_ready_task;
use crate::SEASON;
use log::error;
use log::info;
use log::warn;
use protobuf::Message;
use rand::Rng;
use std::borrow::BorrowMut;
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use tools::cmd_code::{BattleCode, ClientCode, GameCode};
use tools::macros::GetMutRef;
use tools::protos::room::{
    C_CHANGE_TEAM, C_CHOOSE_CHARACTER, C_CHOOSE_SKILL, C_CONFIRM_INTO_ROOM, C_EMOJI, C_KICK_MEMBER,
    C_PREPARE_CANCEL, C_ROOM_SETTING, S_CHOOSE_CHARACTER, S_CHOOSE_CHARACTER_NOTICE,
    S_CHOOSE_SKILL, S_INTO_ROOM_CANCEL_NOTICE, S_LEAVE_ROOM, S_PUNISH_MATCH_NOTICE, S_ROOM,
    S_ROOM_SETTING, S_START,
};
use tools::protos::server_protocol::{
    B_R_G_PUNISH_MATCH, B_R_SUMMARY, G_R_CREATE_ROOM, G_R_JOIN_ROOM, G_R_SEARCH_ROOM,
    R_S_UPDATE_SEASON,
};
use tools::templates::emoji_temp::EmojiTemp;
use tools::util::packet::Packet;

pub fn reload_temps(_: &mut RoomMgr, _: Packet) {
    let path = std::env::current_dir();
    if let Err(e) = path {
        warn!("{:?}", e);
        return;
    }
    let path = path.unwrap();
    let str = path.as_os_str().to_str();
    if let None = str {
        warn!("reload_temps can not path to_str!");
        return;
    }
    let str = str.unwrap();
    let res = str.to_string() + "/template";
    let res = crate::TEMPLATES.reload_temps(res.as_str());
    if let Err(e) = res {
        warn!("{:?}", e);
        return;
    }
    info!("reload_temps success!");
}

///更新赛季
pub fn update_season(rm: &mut RoomMgr, packet: Packet) {
    let mut usn = R_S_UPDATE_SEASON::new();
    let res = usn.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let season_id = usn.get_season_id();
    let next_update_time = usn.get_next_update_time();
    unsafe {
        SEASON.season_id = season_id;
        SEASON.next_update_time = next_update_time;
    }

    //处理更新内存
    let mgr = crate::TEMPLATES.constant_temp_mgr();
    let round_season_id = mgr.temps.get("round_season_id");
    if let None = round_season_id {
        warn!("the constant temp is None!key:round_season_id");
        return;
    }
    let round_season_id = round_season_id.unwrap();
    let res = u32::from_str(round_season_id.value.as_str());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let round_season_id = res.unwrap();
    if round_season_id != season_id {
        return;
    }

    let mut redis_lock = crate::REDIS_POOL.lock().unwrap();
    //更新所有内存数据
    for &user_id in rm.player_room.clone().keys() {
        let room = rm.get_room_mut(&user_id);
        if room.is_none() {
            continue;
        }
        let room = room.unwrap();
        let member = room.get_member_mut(&user_id);
        if member.is_none() {
            continue;
        }
        let member = member.unwrap();
        let rank: Option<String> = redis_lock.hget(
            crate::REDIS_INDEX_RANK,
            crate::REDIS_KEY_CURRENT_RANK,
            user_id.to_string().as_str(),
        );
        if let Some(rank) = rank {
            let ri: RankInfo = serde_json::from_str(rank.as_str()).unwrap();
            member.league = ri.league.into();
        }
    }
}

///创建房间
pub fn create_room(rm: &mut RoomMgr, packet: Packet) {
    //解析gameserver过来的protobuf
    let mut grc = G_R_CREATE_ROOM::new();
    let res = grc.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }

    let room_type = RoomType::try_from(grc.get_room_type() as u8);
    if let Err(e) = room_type {
        error!("{:?}", e);
        return;
    }
    let room_type = room_type.unwrap();
    let user_id = packet.get_user_id();

    match room_type {
        RoomType::OneVOneVOneVOneCustom => {
            //校验这个用户在不在房间内
            let res = rm.get_room_id(&packet.get_user_id());
            if let Some(room_id) = res {
                warn!(
                    "this user already in the room,can not create room! user_id:{},room_id:{}",
                    user_id, room_id
                );
                return;
            }
        }
        RoomType::WorldBossCustom => {
            warn!("this function is not open yet!");
            return;
        }
        _ => {
            warn!("could not create room,the room_type is invalid!");
            return;
        }
    }

    let owner = Member::from(grc.get_pbp());
    let mut room_id: u32 = 0;
    let room_type = RoomType::from(room_type);
    //创建房间
    match room_type {
        RoomType::OneVOneVOneVOneCustom => {
            let res = rm.custom_room.create_room(
                owner,
                rm.get_sender_clone(),
                rm.get_task_sender_clone(),
            );
            match res {
                Ok(id) => room_id = id,
                Err(e) => {
                    warn!("{:?}", e);
                    return;
                }
            }
        }
        _ => {}
    }

    let res = tools::binary::combine_int_2_long(room_type as u32, room_id);
    rm.player_room.insert(packet.get_user_id(), res);
}

//离开房间
pub fn leave_room(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        warn!("could not find room!user_id:{}", user_id);
        return;
    }
    let room = room.unwrap();
    let room_state = room.get_state();
    let room_id = room.get_room_id();
    let room_type = room.get_room_type();
    let member_count = room.get_member_count();
    let member = room.get_member_ref(&user_id);
    //如果成员不在房间，直接退出
    if let None = member {
        warn!(
            "this user is not in the room!user_id:{},room_id:{:?}",
            user_id, room_id
        );
        return;
    }
    let member = member.unwrap();
    let member_state = member.state;

    //如果是匹配放，房间人满，而且未开始战斗，则不允许退出房间
    if room_type == RoomType::OneVOneVOneVOneMatch
        && member_count == MEMBER_MAX as usize
        && room_state == RoomState::AwaitConfirm
    {
        warn!(
            "invalid cmd:leave_room! room_state:{:?},room_id:{},user_id:{}",
            room_state, room_id, user_id
        );
        return;
    }

    //如果是匹配放，房间人满，而且未开始战斗，则不允许退出房间
    if room_type == RoomType::OneVOneVOneVOneMatch
        && member_count == MEMBER_MAX as usize
        && room_state == RoomState::AwaitReady
    {
        warn!(
            "match room is full,could not leave room now! room_id:{},user_id:{}",
            room_id, user_id
        );
        return;
    }

    //房间为等待状态，并且已经准备了，则不允许退出房间
    if room_state == RoomState::AwaitReady && member_state == MemberState::Ready {
        warn!(
            "leave_room:the room is RoomState::Await,this player is already ready!user_id:{}",
            user_id
        );
        return;
    }
    //如果战斗已经开始了,交给战斗服处理
    if room_state == RoomState::ChoiceIndex {
        //如果是匹配房，删除玩家数据，不需要推送，战斗服已经处理过了
        if room_type == RoomType::OneVOneVOneVOneMatch {
            rm.remove_member_without_push(user_id);
        }
        //通知战斗服进行处理
        rm.send_2_server(BattleCode::LeaveRoom.into_u32(), user_id, Vec::new());
        return;
    }
    //不然走正常离开房间流程
    let res = handler_leave_room(rm, user_id, true, false);
    if let Err(e) = res {
        warn!("{:?}", e);
    }
}

///玩家离线
pub fn off_line(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    //校验房间是否存在
    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        //通知游戏服卸载玩家数据
        rm.send_2_server(GameCode::UnloadUser.into_u32(), user_id, Vec::new());
        return;
    }
    let room = room.unwrap();
    let room_id = room.get_room_id();
    let room_state = room.get_state();
    //不在房间就返回
    if !room.members.contains_key(&user_id) {
        warn!(
            "this user is not in the room!user_id:{},room_id:{:?}",
            user_id, room_id
        );
        return;
    }

    //如果房间已经开始战斗则删除玩家不推送，然后通知战斗服
    match room_state {
        RoomState::ChoiceIndex => {
            rm.remove_member_without_push(user_id);
            //通知战斗服
            rm.send_2_server(BattleCode::OffLine.into_u32(), user_id, Vec::new());
        }
        _ => {
            //处理离开房间
            let res = handler_leave_room(rm, user_id, false, true);
            if let Err(e) = res {
                warn!("{:?}", e);
            }
            //通知游戏服卸载玩家数据
            rm.send_2_server(GameCode::UnloadUser.into_u32(), user_id, Vec::new());
        }
    }
}

///取消匹配房间
pub fn cancel_search_room(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        warn!("this user is not matching the room!user_id:{}", user_id);
        return;
    }
    //删除玩家房间数据
    rm.remove_member_without_push(user_id);
    //返回客户端消息
    rm.send_2_client(ClientCode::CancelSearch, user_id, Vec::new());
}

///寻找房间并加入房间
pub fn search_room(rm: &mut RoomMgr, packet: Packet) {
    let mut grs = G_R_SEARCH_ROOM::new();
    let res = grs.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }

    let room_type = RoomType::try_from(grs.get_room_type() as u8);
    if let Err(e) = room_type {
        error!("{:?}", e);
        return;
    }
    let room_type = room_type.unwrap();
    let room_type_u8 = room_type.into_u8();
    let user_id = packet.get_user_id();
    //校验模式
    if room_type_u8 < RoomType::OneVOneVOneVOneMatch.into_u8()
        || room_type_u8 > RoomType::WorldBossCustom.into_u8()
    {
        warn!(
            "search_room:this room type is invaild!room_type:{:?}",
            room_type
        );
        return;
    }
    //校验玩家是否已经在房间里
    if rm.check_player(&user_id) {
        warn!(
            "search_room:this player already in the room!user_id:{}",
            user_id
        );
        return;
    }
    //执行正常流程
    let sender = rm.get_sender_clone();
    let task_sender = rm.get_task_sender_clone();

    let mut member = Member::from(grs.get_pbp());
    member.state = MemberState::AwaitConfirm;
    let punish_match_pt = grs.get_pbp().get_punish_match();
    member.punish_match = PunishMatch::from(punish_match_pt);
    let res = member.reset_punish_match();
    if let Some(pm) = res {
        //推送服务器
        let mut brg = B_R_G_PUNISH_MATCH::new();
        brg.set_punish_match(pm.into());
        let bytes = brg.write_to_bytes();
        match bytes {
            Ok(bytes) => {
                rm.send_2_server(GameCode::SyncPunish.into_u32(), user_id, bytes);
            }
            Err(e) => {
                warn!("{:?}", e);
            }
        }
        //推送给客户端
        let mut proto = S_PUNISH_MATCH_NOTICE::new();
        proto.set_user_id(user_id);
        proto.set_punish_match(pm.into());
        let bytes = proto.write_to_bytes();
        match bytes {
            Ok(bytes) => {
                rm.send_2_client(ClientCode::PunishPatchPush, user_id, bytes);
            }
            Err(e) => {
                warn!("{:?}", e);
            }
        }
    }
    //校验是否允许匹配
    if member.punish_match.punish_id != 0 {
        warn!(
            "search_room:this user could not match now!user_id:{},punish:{:?}",
            user_id, member.punish_match
        );
        return;
    }
    let room_id;
    match room_type {
        RoomType::OneVOneVOneVOneMatch => {
            let match_room = rm.match_room.borrow_mut();
            let res = match_room.quickly_start(member, sender, task_sender);
            //返回错误信息
            if let Err(e) = res {
                warn!("{:?}", e);
                return;
            };
            room_id = res.unwrap();
        }
        RoomType::WorldBossCustom => {
            room_id = 0;
        }
        _ => {
            room_id = 0;
        }
    }

    let value = tools::binary::combine_int_2_long(RoomType::OneVOneVOneVOneMatch as u32, room_id);
    rm.player_room.insert(user_id, value);
}

///准备
pub fn prepare_cancel(rm: &mut RoomMgr, packet: Packet) {
    let mut cpc = C_PREPARE_CANCEL::new();
    let res = cpc.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let user_id = packet.get_user_id();
    let prepare = cpc.prepare;
    let room = rm.get_room_mut(&user_id);
    //校验玩家房间
    if let None = room {
        warn!(
            "prepare_cancel:this player not in the room!user_id:{}",
            user_id
        );
        return;
    }

    let room = room.unwrap();
    let room_id = room.get_room_id();
    let room_type = room.get_room_type();
    let room_state = room.get_state();
    //匹配房，玩家到齐了才可以准备
    if room_type == RoomType::OneVOneVOneVOneMatch && room.get_member_count() != MEMBER_MAX as usize
    {
        warn!(
            "prepare_cancel:this room is not full,so can not prepare!room_id:{}.user_id:{}",
            room_id, user_id
        );
        return;
    }
    //校验房间是否已经开始游戏
    if room_state != RoomState::AwaitReady {
        warn!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        return;
    }
    //校验玩家是否选了角色
    let member = room.members.get(&user_id);
    if let None = member {
        error!("prepare_cancel: this player is None!user_id:{}", user_id);
        return;
    }
    let member = member.unwrap();
    let cter_id = member.chose_cter.cter_id;
    if cter_id == 0 {
        warn!(
            "prepare_cancel: this player has not choose character yet!user_id:{}",
            user_id
        );
        return;
    }

    let cter_temp = crate::TEMPLATES
        .character_temp_mgr()
        .temps
        .get(&cter_id)
        .unwrap();

    //校验玩家是否选了技能
    if prepare && member.chose_cter.skills.len() < cter_temp.usable_skill_count as usize {
        warn!(
            "prepare_cancel: this player has not choose character'skill yet!user_id:{}",
            user_id
        );
        return;
    }
    room.prepare_cancel(&user_id, prepare);
}

///开始游戏
pub fn start(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();

    //校验房间
    let room = rm.get_mut_ref().get_room_mut(&user_id);
    if let None = room {
        warn!("this player is not in the room!user_id:{}", user_id);
        return;
    }
    let room = room.unwrap();

    //校验房间是否已经开始游戏
    if room.is_started() {
        warn!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        return;
    }

    //校验准备状态
    if !room.check_ready() {
        warn!("there is player not ready,can not start game!");
        return;
    }
    //校验是否加载机器人
    check_add_robot(rm.get_mut_ref(), room);
    //执行开始逻辑
    room.start();

    let mut ss = S_START::new();
    ss.is_succ = true;
    room.send_2_client(ClientCode::Start, user_id, ss.write_to_bytes().unwrap());
}

///检查添加机器人
pub fn check_add_robot(rm: &mut RoomMgr, room: &mut Room) {
    //如果没有开启ai则直接return
    if !room.setting.is_open_ai {
        return;
    }
    let need_num = MEMBER_MAX - room.members.len() as u8;
    //需要机器人的数量，为0则直接return
    if need_num == 0 {
        return;
    }
    //机器人模版管理器
    let robot_temp_mgr = crate::TEMPLATES.robot_temp_mgr();
    //角色模版管理器
    let cter_temp_mgr = crate::TEMPLATES.character_temp_mgr();
    //克隆一份机器人角色数组
    let mut cters_res = robot_temp_mgr.cters.clone();
    let mut cters_c = Vec::new();
    //添加已经选择了的角色
    for member in room.members.values() {
        cters_c.push(member.chose_cter.cter_id);
    }
    //删掉已经选择了的角色
    let mut delete_v = Vec::new();
    for i in cters_c.iter() {
        for index in 0..cters_res.len() {
            let j = cters_res.get(index).unwrap();
            if i != j {
                continue;
            }
            delete_v.push(index);
        }
    }
    //删除
    for index in delete_v {
        cters_res.remove(index);
    }
    let mut rand = rand::thread_rng();

    //生成机器人
    for _ in 0..need_num {
        //随机出下标
        let index = rand.gen_range(0..cters_res.len());
        let cter_id = cters_res.remove(index);
        let cter_temp = cter_temp_mgr.get_temp_ref(&cter_id);
        if cter_temp.is_none() {
            error!("could not find this cter!cter_id={}", cter_id);
            continue;
        }
        let cter_temp = cter_temp.unwrap();

        let robot_temp = robot_temp_mgr.get_temp_ref(&cter_id);
        if let None = robot_temp {
            warn!("can not find robot_temp!cter_id:{}", cter_id);
            continue;
        }
        let robot_temp = robot_temp.unwrap();

        //机器人id自增
        crate::ROBOT_ID.fetch_add(1, Ordering::SeqCst);
        let robot_id = crate::ROBOT_ID.load(Ordering::SeqCst);

        //初始化成员
        let mut member = Member::default();
        member.is_robot = true;
        member.user_id = robot_id;
        member.state = MemberState::Ready;
        member.nick_name = "robot".to_owned();

        //初始化选择的角色
        let mut cter = Character::default();
        cter.user_id = robot_id;
        cter.cter_id = cter_id;

        //初始化角色技能
        let skill_count = cter_temp.usable_skill_count;
        for i in 0..skill_count {
            let skill_group = robot_temp.skills.get(i as usize);
            if skill_group.is_none() {
                error!("can not find skill!group:{}", i);
                continue;
            }
            let skill_group = skill_group.unwrap();
            let index = rand.gen_range(0..skill_group.group.len());
            let skill_id = skill_group.group.get(index).unwrap();
            cter.skills.push(*skill_id);
        }
        //将角色加入到成员里
        member.chose_cter = cter;
        let res = room.add_member(member);
        if let Err(e) = res {
            error!("{:?}", e);
            return;
        }
        let room_id = room.get_room_id();
        let value =
            tools::binary::combine_int_2_long(RoomType::OneVOneVOneVOneCustom as u32, room_id);
        rm.player_room.insert(robot_id, value);
    }
}

///换队伍
pub fn change_team(rm: &mut RoomMgr, packet: Packet) {
    let user_id = &packet.get_user_id();

    let mut cct = C_CHANGE_TEAM::new();
    let res = cct.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let team_id = cct.get_target_team_id();
    if team_id < TeamId::Min as u32 || team_id > TeamId::Max as u32 {
        warn!("target_team_id:{} is invaild!", team_id);
        return;
    }
    let room_id = rm.get_room_id(user_id);
    if let None = room_id {
        warn!("this player is not in the room!user_id:{}", user_id);
        return;
    }
    let room_id = room_id.unwrap();
    let room = rm.custom_room.rooms.get_mut(&room_id);
    if let None = room {
        warn!("this player is not in the room!user_id:{}", user_id);
        return;
    }

    let room = room.unwrap();

    //校验房间是否已经开始游戏
    if room.is_started() {
        warn!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        return;
    }
    room.change_team(user_id, &(team_id as u8));
}

///T人
pub fn kick_member(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();

    let mut ckm = C_KICK_MEMBER::new();
    let res = ckm.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let target_id = ckm.target_id;
    //校验房间
    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        warn!(
            "kick_member:this player is not in the room!user_id:{}",
            user_id
        );
        return;
    }

    //校验操作人是不是房主
    let room = room.unwrap();

    //校验房间是否已经开始游戏
    if room.is_started() {
        warn!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        return;
    }

    if room.get_room_type() != RoomType::OneVOneVOneVOneCustom {
        warn!(
            "kick_member:this room is not custom room,can not kick member!room_id:{}",
            room.get_room_id()
        );
        return;
    }

    if room.get_owner_id() != user_id {
        warn!("kick_member:this player is not host!user_id:{}", user_id);
        return;
    }

    //校验房间是否存在target_id这个成员
    if !room.is_exist_member(&target_id) {
        warn!(
            "kick_member:this target player is not in the room!target_user_id:{}",
            target_id
        );
        return;
    }

    let res = room.kick_member(&user_id, &target_id);
    match res {
        Ok(_) => {
            rm.player_room.remove(&target_id);
        }
        Err(e) => {
            warn!("{:?}", e);
            return;
        }
    }
}

///房间设置
pub fn room_setting(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let room = rm.get_room_mut(&user_id);
    let mut srs = S_ROOM_SETTING::new();
    srs.is_succ = true;
    if room.is_none() {
        warn!(
            "room_setting:this player is not in the room,room_id:{}",
            user_id
        );
        return;
    }
    let room = room.unwrap();

    //校验房间是否已经开始游戏
    if room.get_state() != RoomState::AwaitReady {
        warn!(
            "can not setting room!room_id:{},room_state:{:?}",
            room.get_room_id(),
            room.get_state()
        );
    }

    //校验房间是否存在这个玩家
    let member = room.get_member_ref(&user_id);
    if member.is_none() {
        warn!(
            "room_setting:this player is not in the room,room_id:{}",
            user_id
        );
        return;
    }

    let member = member.unwrap();

    //校验玩家是否是房主
    if room.get_owner_id() != user_id {
        warn!(
            "this player is not master:{},room_id:{}",
            user_id,
            room.get_room_id()
        );
        return;
    }
    //校验角色状态
    if member.state == MemberState::Ready {
        warn!("this owner is ready!,user_id:{}", user_id);
        return;
    }

    //走正常逻辑
    if srs.is_succ {
        let mut rs = C_ROOM_SETTING::new();
        let proto_value = rs.get_value();
        let res = rs.merge_from_bytes(packet.get_data());
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
            return;
        }
        let set_type = rs.get_set_type();
        let room_set_type = RoomSettingType::from(set_type);
        match room_set_type {
            RoomSettingType::IsOpenAI => {
                room.setting.is_open_ai = proto_value == 1;
            }
            RoomSettingType::SeasonId => {
                room.setting.season_id = proto_value;
            }
            RoomSettingType::TurnLimitTime => {
                let id = proto_value as u8;
                let limit_time_mgr = crate::TEMPLATES.battle_limit_time_temp_mgr();
                let res = limit_time_mgr.get_temp(&id);
                match res {
                    Ok(temp) => {
                        room.setting.turn_limit_time = temp.ms;
                    }
                    Err(e) => {
                        warn!("{:?}", e);
                        room.setting.turn_limit_time = 60;
                    }
                }
            }
            _ => {
                warn!("room_setting:the proto' value is invalid!");
                return;
            }
        }
    }

    //回给客户端
    room.send_2_client(
        ClientCode::RoomSetting,
        user_id,
        srs.write_to_bytes().unwrap(),
    );
    room.room_notice();
}

///加入房间
pub fn join_room(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let mut grj = G_R_JOIN_ROOM::new();
    let res = grj.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }

    let room_id = grj.room_id;
    //校验玩家是否在房间内
    let res = rm.check_player(&user_id);
    if res {
        warn!("this player already in the room!user_id:{}", user_id);
        return;
    }

    //校验改房间是否存在
    let room = rm.custom_room.get_mut_room_by_room_id(&room_id);
    if let Err(e) = room {
        warn!("{:?}", e);
        return;
    }

    let room = room.unwrap();

    //校验房间是否已经开始游戏
    if room.is_started() {
        warn!(
            "can not leave room,this room is already started!room_id:{}",
            room.get_room_id()
        );
        return;
    }

    let room_type = room.get_room_type();
    //校验房间类型
    if room_type.into_u8() > RoomType::WorldBossCustom.into_u8()
        || room_type == RoomType::OneVOneVOneVOneMatch
    {
        warn!(
            "this room can not join in!room_id:{},room_type:{:?}!",
            room.get_room_id(),
            room_type,
        );
        return;
    }

    //校验房间人数
    if room.members.len() >= MEMBER_MAX as usize {
        warn!("this room already have max player num!,room_id:{}", room_id);
        return;
    }

    // 校验玩家是否在房间里
    let res = room.is_exist_member(&packet.get_user_id());
    if res {
        warn!(
            "this player already in the room!user_id:{},room_id:{}",
            packet.get_user_id(),
            room_id
        );
        return;
    }
    let member = Member::from(grj.get_pbp());
    //将玩家加入到房间
    let res = room.add_member(member);
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let room_id = res.unwrap();
    let value = tools::binary::combine_int_2_long(room.get_room_type() as u32, room_id);
    rm.player_room.insert(user_id, value);
}

///选择角色
pub fn choose_character(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);

    //校验玩家在不在房间
    if res.is_none() {
        warn!("this player is not in room!user_id:{}", user_id);
        return;
    }

    let room = res.unwrap();
    //校验房间状态
    if room.is_started() {
        warn!("this room already started!room_id:{}", room.get_room_id());
        return;
    }

    //解析protobuf
    let mut ccc = C_CHOOSE_CHARACTER::new();
    let res = ccc.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let cter_id = ccc.cter_id;
    if cter_id == 0 {
        warn!(
            "choose_character-the param is error!cter_id=0!user_id:{}",
            user_id
        );
        return;
    }

    let member = room.get_member_ref(&user_id).unwrap();
    //不能发无效的选择
    if member.chose_cter.cter_id == cter_id {
        warn!(
            "choose_character-the param is error!cter_id is repeated!!user_id:{},cter_id:{}",
            user_id, cter_id
        );
        return;
    }

    //校验角色
    let res = room.check_character(user_id, cter_id);
    if let Err(e) = res {
        warn!("{:?}", e);
        return;
    }

    let member = room.get_member_mut(&user_id).unwrap();
    //校验玩家状态
    if member.state == MemberState::Ready {
        warn!("this player is already prepare!user_id:{}", user_id);
        return;
    }

    let cter = member.cters.get(&cter_id);
    //校验角色
    if cter_id > 0 && cter.is_none() {
        warn!(
            "this player do not have this character!user_id:{},cter_id:{}",
            user_id, cter_id
        );
        return;
    }
    if cter.is_some() {
        let cter = cter.unwrap();
        member.chose_cter = cter.clone();
    } else if cter_id == 0 {
        let choice_cter = Character::default();
        member.chose_cter = choice_cter;
    }

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
    let bytes = sccn.write_to_bytes().unwrap();
    let room_mut_ref = room.get_mut_ref();
    for member_id in room.members.keys() {
        room_mut_ref.send_2_client(ClientCode::ChoiceCharacterNotice, *member_id, bytes.clone());
    }
}

///选择技能
pub fn choice_skills(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let mut ccs = C_CHOOSE_SKILL::new();
    let res = ccs.merge_from_bytes(packet.get_data());
    if res.is_err() {
        error!("{:?}", res.err().unwrap());
        return;
    }
    let skills = ccs.get_skills();

    let room = rm.get_room_mut(&user_id);
    if room.is_none() {
        warn!("this player is not in room!user_id:{}", user_id);
        return;
    }

    let room = room.unwrap();
    let room_state = room.get_state();
    let member = room.get_member_mut(&user_id).unwrap();
    if member.chose_cter.cter_id == 0 {
        warn!(
            "this player not choice cter yet!can not choice skill of cter!user_id:{}",
            user_id
        );
        return;
    }

    let cter_id = member.chose_cter.cter_id;

    let cter = member.cters.get(&cter_id).unwrap();

    //校验房间壮体啊
    if room_state != RoomState::AwaitReady {
        warn!("can not choice skill now!");
        return;
    }

    //校验成员状态
    if member.state == MemberState::Ready {
        warn!(
            "this player already ready,can not choice skill now!user_id:{}",
            user_id
        );
        return;
    }

    let cter_temp = crate::TEMPLATES
        .character_temp_mgr()
        .get_temp_ref(&cter_id)
        .unwrap();
    //校验技能数量
    if skills.len() > cter_temp.usable_skill_count as usize {
        warn!("this cter's skill count is error! cter_id:{}", cter_id);
        return;
    }
    //校验技能有效性
    for skill in skills.iter() {
        if !cter.skills.contains(skill) {
            warn!(
                "this cter do not have this skill!user_id:{},cter_id:{},skill_id:{}",
                user_id, cter_id, *skill
            );
            return;
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
                warn!("the skill group is error!user_id:{}", user_id);
                return;
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
}

///确认进入房间
pub fn confirm_into_room(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    //校验玩家在不在房间内
    let room = rm.get_room_mut(&user_id);
    if let None = room {
        warn!("this user is not in the room!");
        return;
    }
    let room = room.unwrap();
    let room_type = room.get_room_type();
    let room_id = room.get_room_id();
    //校验房间类型
    if room.get_room_type() != RoomType::OneVOneVOneVOneMatch {
        warn!(
            "this room is not Match Room!room_type:{:?},room_id:{}",
            room_type, room_id
        );
        return;
    }

    let mut ccir = C_CONFIRM_INTO_ROOM::new();
    let res = ccir.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let confirm = ccir.confirm;

    //如果房间成员不满，不允许发这个命令
    if room.members.len() < MEMBER_MAX as usize {
        warn!(
            "this room is not full,could not handler confirm!room_type:{:?},room_id:{}",
            room_type, room_id
        );
        return;
    }

    let room_state = room.state;
    //校验房间状态
    if room_state != RoomState::AwaitConfirm {
        warn!(
            "the Match Room state is {:?}!room_id:{}",
            room_state, room_id
        );
        return;
    }

    //如果全部确认进入房间，就发送通知房间协议给所有客户端
    if confirm {
        //推送确认进入人数
        room.notice_confirm_count(user_id);

        //通知新成员加入
        room.notice_new_member(user_id);

        //判断人是否满了，满了就把房间信息推送给客户端
        let res = room.check_all_confirmed_into_room();
        if res {
            room.state = RoomState::AwaitReady;
            let mut sr = S_ROOM::new();
            sr.is_succ = true;
            sr.set_room(room.convert_to_pt());
            let bytes = sr.write_to_bytes().unwrap();
            room.send_2_all_client(ClientCode::Room, bytes);
            let task_sender = rm.get_task_sender_clone();
            build_match_room_ready_task(room_id, task_sender);
        }
    } else if room.state == RoomState::AwaitConfirm {
        //解散房间，并通知所有客户端
        let sircn = S_INTO_ROOM_CANCEL_NOTICE::new();
        let bytes = sircn.write_to_bytes().unwrap();
        room.send_2_all_client(ClientCode::IntoRoomCancelNotice, bytes);
        //删除房间
        rm.rm_room_without_push(room_type, room_id);
    }
}

///结算
pub fn summary(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    if user_id == 0 {
        warn!("summary,the user_id is 0!");
        return;
    }
    let room = rm.get_room_mut(&user_id);
    if let None = room {
        return;
    }
    let room = room.unwrap();
    let room_type = room.get_room_type();
    let room_id = room.get_room_id();

    match room_type {
        //如果是匹配房，直接删除房间数据
        RoomType::OneVOneVOneVOneMatch => {
            rm.rm_room_without_push(room_type, room_id);
        }
        //如果是自定义房间，更新结算数据
        RoomType::OneVOneVOneVOneCustom => {
            //如果是自定义房间
            room.state = RoomState::AwaitReady;
            let mut brs = B_R_SUMMARY::new();
            let res = brs.merge_from_bytes(packet.get_data());
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            for sd in brs.summary_datas.iter() {
                let user_id = sd.user_id;
                let member = room.get_member_mut(&user_id);
                if let None = member {
                    continue;
                }
                let member = member.unwrap();
                member.chose_cter = Character::default();
                member.grade = sd.grade as u8;
                member.league.update(
                    sd.get_league().get_league_id() as i8,
                    sd.get_league().league_score,
                    sd.get_league().league_time,
                );
                member.state = MemberState::NotReady;
            }
        }
        _ => {}
    }
}

///发送表情
pub fn emoji(rm: &mut RoomMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);
    if res.is_none() {
        warn!("this player is not in the room!user_id:{}", user_id);
        return;
    }
    let room = res.unwrap();
    //如果战斗已经开始,则转发给战斗服
    if room.is_started() {
        room.send_2_server(BattleCode::Emoji.into_u32(), user_id, packet.get_data_vec());
        return;
    }
    let member = room.get_member_mut(&user_id);
    if member.is_none() {
        warn!("this player is not in the room!user_id:{}", user_id);
        return;
    }
    let member = member.unwrap();
    if member.state != MemberState::Ready {
        warn!(
            "this player is not ready,can not send emoji!user_id:{}",
            user_id
        );
        return;
    }

    let mut ce = C_EMOJI::new();
    let res = ce.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let emoji_id = ce.emoji_id;
    let res: Option<&EmojiTemp> = crate::TEMPLATES.emoji_temp_mgr().temps.get(&emoji_id);
    if res.is_none() {
        warn!("there is no temp for emoji_id:{}", emoji_id);
        return;
    }
    //校验表情是否需要解锁和角色表情
    let emoji = res.unwrap();
    if emoji.condition != 0 {
        warn!("this emoji need unlock!emoji_id:{}", emoji_id);
        return;
    } else if emoji.condition == 0
        && emoji.cter_id > 0
        && emoji.cter_id != member.chose_cter.cter_id
    {
        warn!(
            "this character can not send this emoji!cter_id:{},emoji_id:{}",
            member.chose_cter.cter_id, emoji_id
        );
        return;
    }
    //走正常逻辑
    room.emoji(user_id, emoji_id);
}

///处理离开房间
pub fn handler_leave_room(
    rm: &mut RoomMgr,
    user_id: u32,
    need_push_self: bool,
    need_punish: bool,
) -> anyhow::Result<()> {
    let room = rm.get_room_mut(&user_id).unwrap();
    let room_id = room.get_room_id();
    let room_type = RoomType::from(room.get_room_type());

    //处理退出房间
    let need_rm = match room_type {
        RoomType::OneVOneVOneVOneCustom => {
            let res = rm.custom_room.leave_room(
                MemberLeaveNoticeType::Leave as u8,
                &room_id,
                &user_id,
                need_push_self,
                false,
            );
            if let Err(e) = res {
                error!("{:?}", e);
                return Ok(());
            }

            let room = rm.custom_room.rooms.get(&room_id).unwrap();
            let owner_id = room.get_owner_id();
            if room.is_empty() || user_id == owner_id {
                true
            } else {
                false
            }
        }
        RoomType::OneVOneVOneVOneMatch => {
            let room = rm.match_room.rooms.get_mut(&room_id).unwrap();
            if room.is_empty() {
                return Ok(());
            }

            let res = rm.match_room.leave_room(
                MemberLeaveNoticeType::Leave as u8,
                &room_id,
                &user_id,
                need_push_self,
                need_punish,
            );
            if let Err(e) = res {
                error!("{:?}", e);
                return Ok(());
            }
            let mut slr = S_LEAVE_ROOM::new();
            slr.set_is_succ(true);
            if need_push_self {
                rm.send_2_client(
                    ClientCode::LeaveRoom,
                    user_id,
                    slr.write_to_bytes().unwrap(),
                );
            }
            let room = rm.match_room.rooms.get(&room_id).unwrap();
            if room.is_empty() {
                true
            } else {
                false
            }
        }
        _ => false,
    };

    //删掉当前离开的玩家
    rm.player_room.remove(&user_id);
    info!(
        "玩家离开{:?}房间，卸载玩家房间数据!user_id:{},room_id:{}",
        room_type, user_id, room_id
    );
    if need_rm {
        rm.rm_room_without_push(room_type, room_id);
    }

    Ok(())
}
