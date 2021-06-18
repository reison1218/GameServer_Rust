use crate::battle::battle_enum::{ActionType, PosType, SkillConsumeType};
use crate::battle::battle_player::BattlePlayer;
use crate::battle::battle_skill::Skill;
use crate::battle::market::handler_buy;
use crate::mgr::battle_mgr::BattleMgr;
use crate::mgr::RankInfo;
use crate::room::map_data::MapCellType;
use crate::room::room::Room;
use crate::room::MemberLeaveNoticeType;
use crate::room::RoomState;
use crate::SEASON;
use log::{error, info, warn};
use protobuf::Message;
use std::borrow::{Borrow, BorrowMut};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::str::FromStr;
use tools::cmd_code::{ClientCode, GameCode};
use tools::protos::base::ActionUnitPt;
use tools::protos::battle::C_BUY;
use tools::protos::battle::{C_ACTION, C_CHOOSE_INDEX, C_POS, S_ACTION_NOTICE, S_POS_NOTICE};
use tools::protos::room::C_EMOJI;
use tools::protos::server_protocol::{R_B_START, R_S_UPDATE_SEASON};
use tools::templates::emoji_temp::EmojiTemp;
use tools::util::packet::Packet;

///购买
pub fn buy(bm: &mut BattleMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let mut buy_proto = C_BUY::new();
    let res = buy_proto.merge_from_bytes(packet.get_data());
    if let Err(err) = res {
        error!("{:?}", err);
        return;
    }
    let room = bm.get_room_mut(&user_id);
    if let None = room {
        error!("this player is not in the room!user_id:{}", user_id);
        return;
    }
    let merchandise_id = buy_proto.merchandise_id;
    let room = room.unwrap();
    //校验房间状态
    if room.state != RoomState::BattleStarted {
        error!(
            "battle is not start!could not buy!user_id:{},room_state:{:?}",
            user_id, room.state
        );
        return;
    }
    let battle_data = &mut room.battle_data;
    let turn_user = battle_data.get_turn_user(None);
    let battle_player = battle_data.battle_player.get_mut(&user_id).unwrap();
    let cter_id = battle_player.get_cter_id();
    //校验玩家死了没
    if battle_player.is_died() {
        return;
    }
    //校验选了站位没
    let cter_index = battle_player.cter.index_data.map_cell_index;
    if cter_index.is_none() {
        error!("this player index is None!user_id:{}", user_id);
        return;
    }
    //校验玩家位置
    let cter_index = cter_index.unwrap();
    if cter_index != battle_data.tile_map.market_cell.0 {
        error!("this player index is not market!user_id:{}", user_id);
        return;
    }
    //校验玩家回合
    match turn_user {
        Ok(turn_user) => {
            if turn_user != user_id {
                warn!(
                    "could not buy!turn_user != user_id,turn_user:{},user_id:{}",
                    turn_user, user_id
                );
                return;
            }
        }
        Err(e) => {
            error!("{:?}", e);
            return;
        }
    }
    let merchandise_temp = crate::TEMPLATES.merchandise_temp_mgr();
    let temp = merchandise_temp.get_temp(&merchandise_id);
    if let Err(e) = temp {
        error!("{:?}", e);
        return;
    }
    let merchandise_temp = temp.unwrap();
    let turn_limit_buy_times = merchandise_temp.turn_limit_buy_times;
    //校验是否可以购买
    let buy_times = battle_player
        .merchandise_data
        .get_turn_buy_times(merchandise_id);
    if buy_times >= turn_limit_buy_times {
        warn!("could not buy this merchandise!turn_limit_buy_times:{},user_id:{},user_turn_buy_times:{}",user_id,turn_limit_buy_times,buy_times);
        return;
    }

    let price = merchandise_temp.price;
    let room_type = battle_data.room_type.into_u8();
    //校验房间类型
    if !merchandise_temp.room_type.contains(&room_type) {
        warn!(
            "the room_type is error!merchandis_id:{},room_type:{:?};cter_id:{} room_type:{}",
            merchandise_id,
            merchandise_temp.room_type,
            battle_player.get_cter_id(),
            room_type
        );
        return;
    }
    let cter_temp_mgr = crate::TEMPLATES.character_temp_mgr();
    let cter_temp = cter_temp_mgr.temps.get(&cter_id).unwrap();

    //匹配角色类型是否相同
    if !is_same(
        merchandise_temp.character_type.as_slice(),
        cter_temp.character_type.as_slice(),
    ) {
        warn!(
            "the character_type is error! merchandis_id:{} ,character_type:{:?};cter_id:{},character_type:{:?}",
            merchandise_id,merchandise_temp.character_type, cter_id,cter_temp.character_type
        );
        return;
    }
    //校验金币是否足够
    if battle_player.gold < price {
        warn!(
            "this player's gold is not enough!merchandis_id:{}'s price is {},player's gold is {}",
            merchandise_id, price, battle_player.gold
        );
        return;
    }
    //执行购买
    handler_buy(battle_data, user_id, merchandise_id);
}

///行动请求
#[track_caller]
pub fn action(bm: &mut BattleMgr, packet: Packet) {
    let rm_ptr = bm as *mut BattleMgr;
    let user_id = packet.get_user_id();
    let res = bm.get_room_mut(&user_id);
    if let None = res {
        warn!("the player is not in the room!user_id:{}", user_id);
        return;
    }
    let room = res.unwrap();
    //校验房间状态
    if room.get_state() != RoomState::BattleStarted {
        warn!(
            "room state is not battle_started!room_id:{},state:{:?}",
            room.get_room_id(),
            room.state
        );
        return;
    }

    //校验用户
    let res = check_user(room, user_id);
    if !res {
        return;
    }

    //解析protobuf
    let mut ca = C_ACTION::new();
    let res = ca.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    //客户端请求的actiontype
    let action_type = ca.get_action_type();
    //客户端请求对应的目标
    let target_index = ca.target_index;
    //客户端请求target对应的值
    let value = ca.value;

    //行动方actionunitpt
    let mut au = ActionUnitPt::new();
    au.action_value.push(ca.value);
    au.from_user = user_id;

    let res;
    let action_type = ActionType::try_from(action_type as u8).unwrap();
    //行为分支
    match action_type {
        //无意义分支
        ActionType::None => {
            warn!("action_type is 0!");
            return;
        }
        //使用道具
        ActionType::UseItem => {
            au.action_type = ActionType::UseItem as u32;
            res = use_item(room, user_id, value, target_index, &mut au);
        }
        //跳过
        ActionType::Skip => {
            au.action_type = ActionType::Skip as u32;
            unsafe {
                res = skip_turn(rm_ptr.as_mut().unwrap(), user_id, &mut au);
            }
        }
        //翻地图块
        ActionType::Open => {
            au.action_type = ActionType::Open as u32;
            res = open_map_cell(room, user_id, value as usize, &mut au);
        }
        //使用技能
        ActionType::Skill => {
            au.action_type = ActionType::Skill as u32;
            res = use_skill(room, user_id, value, target_index, &mut au);
        }
        //普通攻击
        ActionType::Attack => {
            au.action_type = ActionType::Attack as u32;
            res = attack(room, user_id, target_index, &mut au);
        }
        //解锁
        ActionType::EndShowMapCell => {
            au.action_type = ActionType::EndShowMapCell as u32;
            res = unlock_oper(room, user_id, &mut au);
        }
        _ => {
            warn!("action_type is error!action_type:{:?}", action_type);
            return;
        }
    }

    //如果有问题就返回
    if let Err(e) = res {
        warn!("{:?}", e);
        return;
    }

    //回给客户端,添加action主动方
    let mut san_push_all = S_ACTION_NOTICE::new();
    san_push_all.action_uints.push(au);
    let mut san_assign = vec![];
    //添加额外的行动方
    if let Ok(au_vec) = res {
        if let Some(v) = au_vec {
            for (user_id, au) in v {
                if user_id == 0 {
                    san_push_all.action_uints.push(au);
                } else {
                    let mut san = S_ACTION_NOTICE::new();
                    san.action_uints.push(au);
                    san_assign.push((user_id, san));
                }
            }
        }
    }

    //以下通知客户端结果
    let bytes = san_push_all.write_to_bytes();
    if let Err(e) = bytes {
        error!("{:?}", e);
        return;
    }
    let bytes = bytes.unwrap();

    //推送给所有房间成员
    room.send_2_all_client(ClientCode::ActionNotice, bytes);

    if !san_assign.is_empty() {
        for (user_id, san) in san_assign {
            let bytes = san.write_to_bytes();
            match bytes {
                Ok(bytes) => {
                    room.send_2_client(ClientCode::ActionNotice, user_id, bytes);
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
    }

    unsafe {
        let battle_player = room.battle_data.get_battle_player(None, false).unwrap();
        let current_cter_is_died = battle_player.is_died();
        //如果角色没死，并且是机器人，则通知机器人执行完了,并且启动机器人action
        if !current_cter_is_died || battle_player.robot_data.is_some() {
            battle_player.robot_start_action();
        }
        //判断是否进行结算
        let is_summary = process_summary(rm_ptr.as_mut().unwrap(), room);
        if !is_summary && current_cter_is_died {
            room.battle_data.next_turn(true);
        } else if action_type == ActionType::Skip && room.state == RoomState::BattleStarted {
            room.battle_data.send_battle_turn_notice();
        }
    }
}

///处理战斗结算
/// 在action末尾处,用于处理战斗推进过程中的战斗结算
pub unsafe fn process_summary(bm: &mut BattleMgr, room: &mut Room) -> bool {
    let is_summary = room.battle_summary();
    let room_id = room.get_room_id();
    //如果要结算,卸载数据
    if !is_summary {
        return false;
    }
    bm.rm_room(room_id);
    true
}

///开始战斗
pub fn start(bm: &mut BattleMgr, packet: Packet) {
    let mut rbs = R_B_START::new();
    let res = rbs.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        warn!("{:?}", e);
        return;
    }

    let rt = rbs.get_room_pt();
    let tcp_sender = bm.get_game_center_channel_clone();
    let task_sender = bm.get_task_sender_clone();
    let robot_task_sender = bm.get_robot_task_sender_clone();
    //创建战斗房间
    let room = Room::new(rt, tcp_sender, task_sender, robot_task_sender);
    if let Err(e) = room {
        error!("create battle room fail!{:?}", e);
        return;
    }
    let mut room = room.unwrap();
    let room_type = room.get_room_type();
    //开始战斗
    room.start();
    let room_id = room.get_room_id();
    for user_id in room.members.keys() {
        bm.player_room.insert(*user_id, room_id);
    }
    bm.rooms.insert(room.get_room_id(), room);

    info!(
        "房间战斗开始！room_type:{:?},room_id:{}",
        room_type, room_id
    );
}

///处理pos
pub fn pos(rm: &mut BattleMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);
    if let None = res {
        return;
    }
    let room = res.unwrap();
    //校验房间状态
    if room.get_state() != RoomState::BattleStarted {
        warn!(
            "room state is not battle_started!room_id:{}",
            room.get_room_id()
        );
        return;
    }

    let battle_player = room.get_battle_player_mut_ref(&user_id);
    if battle_player.is_none() {
        warn!("battle_player is not find!user_id:{}", user_id);
        return;
    }
    let battle_player = battle_player.unwrap();

    let mut cp = C_POS::new();
    let res = cp.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let skill_id = cp.skill_id;
    let pos_type = cp.field_type;
    //校验技能
    let res = battle_player.cter.skills.contains_key(&skill_id);
    if !res {
        warn!(
            "this battle_cter has no this skill!cter_id:{},skill_id:{}",
            battle_player.get_cter_id(),
            skill_id
        );
        return;
    }
    //校验操作类型
    if pos_type < PosType::ChangePos.into_u32() || pos_type > PosType::CancelPos.into_u32() {
        warn!(
            "the pos_type is error!user_id:{},pos_type:{}",
            user_id, pos_type
        );
        return;
    }
    let mut spn = S_POS_NOTICE::new();
    spn.set_user_id(user_id);
    spn.set_field_type(pos_type);
    spn.set_skill_id(skill_id);
    let bytes = spn.write_to_bytes().unwrap();
    room.send_2_all_client(ClientCode::PosNotice, bytes);
}

///使用道具
fn use_item(
    rm: &mut Room,
    user_id: u32,
    item_id: u32,
    targets: Vec<u32>,
    au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<(u32, ActionUnitPt)>>> {
    let battle_player = rm.battle_data.battle_player.get(&user_id);
    if let None = battle_player {
        let str = format!("battle_cter is not find!user_id:{}", user_id);
        warn!("{:?}", str);
        anyhow::bail!(str)
    }
    let battle_player = battle_player.unwrap();

    if battle_player.is_locked() {
        let str = format!("battle_cter is locked!user_id:{}", user_id);
        warn!("{:?}", str);
        anyhow::bail!(str)
    }

    if battle_player.cter.items.is_empty() {
        let str = format!("battle_cter is not find!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    let res = battle_player.cter.items.contains_key(&item_id);
    if !res {
        let str = format!(
            "this user not have this item!item_id:{},user_id:{}",
            item_id, user_id
        );
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    let res = rm.battle_data.use_item(user_id, item_id, targets, au)?;
    Ok(res)
}

///翻地图块
fn open_map_cell(
    rm: &mut Room,
    user_id: u32,
    target_map_cell_index: usize,
    au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<(u32, ActionUnitPt)>>> {
    let battle_data = rm.battle_data.borrow();
    let battle_player = rm.battle_data.battle_player.get(&user_id).unwrap();
    let cter_index = battle_player.get_map_cell_index();
    if battle_player.is_locked() {
        let str = format!("battle_cter is locked!user_id:{}", user_id);
        warn!("{:?}", str);
        anyhow::bail!(str)
    }
    //校验地图块
    let res =
        battle_data.check_choice_index(target_map_cell_index, false, true, true, true, true, false);
    if let Err(e) = res {
        let str = format!("{:?}", e);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    //校验剩余翻块次数
    if battle_player.flow_data.residue_movement_points <= 0 {
        let str = format!("this player's residue_open_times is 0!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }

    //校验本turn是否翻过目标地图块
    if battle_player
        .flow_data
        .open_map_cell_vec_history
        .contains(&target_map_cell_index)
    {
        let str = format!(
            "this player already has open this map_cell!user_id:{},open_map_cell_vec:{:?},index:{}",
            user_id, battle_player.flow_data.open_map_cell_vec_history, target_map_cell_index
        );
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    let map_cell = battle_data
        .tile_map
        .map_cells
        .get(target_map_cell_index)
        .unwrap();
    let map_cell_type = map_cell.cell_type;
    let map_cell = battle_data.tile_map.map_cells.get(cter_index).unwrap();
    //如果是在同一个位置，然后又是商店则返回
    if map_cell.index == target_map_cell_index && map_cell_type == MapCellType::MarketCell {
        let str = "this player already at the market!".to_owned();
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }

    let battle_data = rm.battle_data.borrow_mut();
    let res = battle_data.open_map_cell(target_map_cell_index, au);
    match res {
        Ok(res) => Ok(res),
        Err(e) => anyhow::bail!(e),
    }
}

///进行普通攻击
fn attack(
    rm: &mut Room,
    user_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<(u32, ActionUnitPt)>>> {
    //先校验玩家是否可以进行攻击
    let battle_player = rm.battle_data.battle_player.get(&user_id).unwrap();
    if battle_player.is_locked() {
        let str = format!("battle_cter is locked!user_id:{}", user_id);
        warn!("{:?}", str);
        anyhow::bail!(str)
    }
    if !battle_player.is_can_attack() {
        let str = format!("now can not attack!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    //如果目标为空
    if target_array.is_empty() {
        let str = format!("the target_array is empty!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    unsafe {
        rm.battle_data.attack(user_id, target_array, au)?;
        Ok(None)
    }
}

///使用技能
fn use_skill(
    rm: &mut Room,
    user_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<(u32, ActionUnitPt)>>> {
    //校验技能id有效性
    let battle_player = rm.battle_data.battle_player.get_mut(&user_id).unwrap();
    if battle_player.is_locked() {
        let str = format!("battle_cter is locked!user_id:{}", user_id);
        warn!("{:?}", str);
        anyhow::bail!(str)
    }
    let skill = battle_player.cter.skills.get(&skill_id);
    if skill.is_none() {
        warn!("this skill is none!skill_id:{}", user_id);
        anyhow::bail!("")
    }
    //校验技能可用状态
    let skill = skill.unwrap();
    let res = check_skill_useable(battle_player, skill);
    if let Err(e) = res {
        warn!("{:?}", e);
        anyhow::bail!("")
    }
    //使用技能，走正常逻辑
    let res = rm
        .battle_data
        .use_skill(user_id, skill_id, false, target_array, au)?;

    Ok(res)
}

///跳过选择回合顺序
fn skip_turn(
    rmgr: &mut BattleMgr,
    user_id: u32,
    _au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<(u32, ActionUnitPt)>>> {
    let rm = rmgr.get_room_mut(&user_id).unwrap();
    //校验现在能不能跳过
    let next_user = rm.get_turn_user(None);
    if let Err(e) = next_user {
        error!("{:?}", e);
        anyhow::bail!("")
    }
    //如果不是下一个turn的，不让跳过
    let next_user = next_user.unwrap();
    if next_user != user_id {
        warn!(
            "skip_choice_turn next_user!=user_id! next_user:{},user_id:{}",
            next_user, user_id
        );
        anyhow::bail!("")
    }

    //拿到战斗角色
    let battle_player = rm.battle_data.battle_player.get(&user_id);
    if battle_player.is_none() {
        warn!("skip_choice_turn battle_cter is none!user_id:{}", user_id);
        anyhow::bail!("")
    }

    //没有翻过地图块，则跳过
    let battle_player = battle_player.unwrap();
    if !battle_player.get_is_can_end_turn() {
        warn!("this player not open any map_cell yet!user_id:{}", user_id);
        anyhow::bail!("")
    }
    let need_refresh_map = rm.battle_data.check_refresh_map();
    //如果需要刷新地图，走地图刷新next turn逻辑
    if need_refresh_map {
        rm.battle_data.choice_index_next_turn();
        rm.refresh_map();
    } else {
        //否则走战斗next turn逻辑
        rm.battle_data.next_turn(false);
    }
    Ok(None)
}

///发送表情
pub fn emoji(bm: &mut BattleMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let res = bm.get_room_mut(&user_id);
    if res.is_none() {
        warn!("this player is not in the room!user_id:{}", user_id);
        return;
    }
    let room = res.unwrap();
    let battle_player = room.battle_data.battle_player.get(&user_id);
    if let None = battle_player {
        warn!("can not find cter!user_id:{}", user_id);
        return;
    }
    let battle_player = battle_player.unwrap();
    let cter_id = battle_player.get_cter_id();

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
    } else if emoji.condition == 0 && emoji.cter_id > 0 && emoji.cter_id != cter_id {
        warn!(
            "this character can not send this emoji!cter_id:{},emoji_id:{}",
            cter_id, emoji_id
        );
        return;
    }
    //走正常逻辑
    room.emoji(user_id, emoji_id);
}

///离线
pub fn off_line(bm: &mut BattleMgr, packet: Packet) {
    let user_id = packet.get_user_id();

    //校验用户不在战斗房间里
    let room = bm.get_room_mut(&user_id);
    if let Some(room) = room {
        let room_id = room.get_room_id();
        //处理玩家离开
        bm.handler_leave(room_id, MemberLeaveNoticeType::OffLine, user_id, false);
    }
    //通知游戏服卸载玩家数据
    bm.send_2_server(GameCode::UnloadUser.into_u32(), user_id, Vec::new());
}

///离开房间
pub fn leave_room(bm: &mut BattleMgr, packet: Packet) {
    let user_id = packet.get_user_id();

    //校验用户不在战斗房间里
    let room = bm.get_room_mut(&user_id);
    if room.is_none() {
        return;
    }
    let room_id = room.unwrap().get_room_id();
    //处理玩家离开
    bm.handler_leave(room_id, MemberLeaveNoticeType::Leave, user_id, true);
}

pub fn reload_temps(_: &mut BattleMgr, _: Packet) {
    let path = std::env::current_dir();
    if let Err(e) = path {
        error!("{:?}", e);
        return;
    }
    let path = path.unwrap();
    let str = path.as_os_str().to_str();
    if let None = str {
        error!("reload_temps can not path to_str!");
        return;
    }
    let str = str.unwrap();
    let res = str.to_string() + "/template";
    let res = crate::TEMPLATES.reload_temps(res.as_str());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    info!("reload_temps success!");
}

///更新赛季
pub fn update_season(bm: &mut BattleMgr, packet: Packet) {
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
    let res = i32::from_str(round_season_id.value.as_str());
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
    for &user_id in bm.player_room.clone().keys() {
        let room = bm.get_room_mut(&user_id);
        if room.is_none() {
            continue;
        }
        let room = room.unwrap();
        let member = room.members.get_mut(&user_id);
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

///选择初始占位
pub fn choice_index(bm: &mut BattleMgr, packet: Packet) {
    let user_id = packet.get_user_id();
    let mut ccl = C_CHOOSE_INDEX::new();
    let res = ccl.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let index = ccl.index;

    let room = bm.get_room_mut(&user_id);
    if room.is_none() {
        warn!("this player is not in the room!user_id:{}", user_id);
        return;
    }
    let room = room.unwrap();

    //校验是否轮到他了
    if !room.is_can_choice_index_now(user_id) {
        warn!(
            "this player is not the next choice index player!user_id:{},index:{},choice_order:{:?}",
            user_id,
            room.get_next_turn_index(),
            room.battle_data.turn_orders
        );
        return;
    }

    let res = room.battle_data.check_choice_index(
        index as usize,
        false,
        false,
        false,
        true,
        false,
        false,
    );
    //校验参数
    if let Err(e) = res {
        warn!("{:?}", e);
        return;
    }

    //校验他选过没有
    let member = room.get_battle_player_ref(&user_id).unwrap();
    if member.cter.map_cell_index_is_choiced() {
        warn!("this player is already choice index!user_id:{}", user_id);
        return;
    }
    room.choice_index(user_id, index);
}

///结束操作
fn unlock_oper(
    rm: &mut Room,
    user_id: u32,
    au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<(u32, ActionUnitPt)>>> {
    //先校验玩家是否可以进行攻击
    let battle_player = rm.battle_data.battle_player.get_mut(&user_id).unwrap();
    let v = *au.action_value.get(0).unwrap();
    if battle_player.status.locked_oper == 0 || battle_player.status.locked_oper != v {
        anyhow::bail!("{there is no show cell skill activate!}")
    }
    battle_player.status.locked_oper = 0;
    battle_player.set_is_can_end_turn(true);
    Ok(None)
}

///校验玩家
fn check_user(rm: &Room, user_id: u32) -> bool {
    let battle_player = rm.get_battle_player_ref(&user_id);
    if let None = battle_player {
        warn!(
            "user is not in the room!room_id:{},user_id:{}",
            rm.get_room_id(),
            user_id
        );
        return false;
    }
    let battle_player = battle_player.unwrap();
    //校验角色是否死亡
    if battle_player.is_died() {
        warn!(
            "this cter is already dead!room_id:{},user_id:{}",
            rm.get_room_id(),
            user_id
        );
        return false;
    }
    //校验是否选择了占位
    if battle_player.cter.index_data.map_cell_index.is_none() {
        warn!(
            "user is not choice index!room_id:{},user_id:{}",
            rm.get_room_id(),
            user_id
        );
        return false;
    }
    let next_user = rm.get_turn_user(None);
    if let Err(e) = next_user {
        error!("{:?}", e);
        return false;
    }
    let next_user = next_user.unwrap();
    if next_user != user_id {
        warn!(
            "next_user is not this user!next_user:{},user_id:{}",
            next_user, user_id
        );
        return false;
    }
    true
}

///校验技能可用状态
fn check_skill_useable(battle_player: &BattlePlayer, skill: &Skill) -> anyhow::Result<()> {
    //校验cd
    if skill.skill_temp.consume_type != SkillConsumeType::Energy.into_u8() && skill.cd_times > 0 {
        anyhow::bail!(
            "this skill cd is not ready!cter_id:{},skill_id:{},cd:{}",
            battle_player.get_cter_id(),
            skill.id,
            skill.cd_times
        )
    } else if skill.skill_temp.consume_type == SkillConsumeType::Energy.into_u8()
        && battle_player.cter.base_attr.energy < skill.skill_temp.consume_value
    {
        anyhow::bail!(
            "this cter's energy is not enough!cter_id:{},skill_id:{},energy:{},cost_energy:{}",
            battle_player.get_cter_id(),
            skill.id,
            battle_player.cter.base_attr.energy,
            skill.skill_temp.consume_value
        )
    }
    Ok(())
}

pub trait Find<T: Clone + Debug> {
    fn find(&self, key: usize) -> Option<&T>;

    fn find_mut(&mut self, key: usize) -> Option<&mut T>;
}

impl Find<Skill> for Vec<Skill> {
    fn find(&self, key: usize) -> Option<&Skill> {
        for value in self.iter() {
            if value.id != key as u32 {
                continue;
            }
            return Some(value);
        }
        None
    }

    fn find_mut(&mut self, key: usize) -> Option<&mut Skill> {
        for value in self.iter_mut() {
            if value.id != key as u32 {
                continue;
            }
            return Some(value);
        }
        None
    }
}

pub trait Delete<T: Clone + Debug> {
    fn delete(&mut self, key: usize);
}

impl Delete<Skill> for Vec<Skill> {
    fn delete(&mut self, key: usize) {
        for index in 0..self.len() {
            let res = self.find(key);
            if res.is_none() {
                continue;
            }
            self.remove(index);
            break;
        }
    }
}

///判断两个vec是否相同
pub fn is_same(vec1: &[u8], vec2: &[u8]) -> bool {
    for &i in vec1.iter() {
        for &j in vec2.iter() {
            if i != j {
                return false;
            }
        }
    }
    true
}
