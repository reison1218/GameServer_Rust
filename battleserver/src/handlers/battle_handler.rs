use crate::battle::battle_enum::skill_type::PUSH_SELF;
use crate::battle::battle_enum::{ActionType, PosType, SkillConsumeType};
use crate::battle::battle_skill::Skill;
use crate::mgr::battle_mgr::BattleMgr;
use crate::room::character::BattleCharacter;
use crate::room::room::Room;
use crate::room::RoomState;
use crate::SEASON;
use log::{error, info, warn};
use protobuf::Message;
use std::borrow::{Borrow, BorrowMut};
use std::convert::TryFrom;
use std::fmt::Debug;
use tools::cmd_code::{ClientCode, GameCode};
use tools::protos::base::ActionUnitPt;
use tools::protos::battle::{C_ACTION, C_CHOOSE_INDEX, C_POS, S_ACTION_NOTICE, S_POS_NOTICE};
use tools::protos::room::C_EMOJI;
use tools::protos::server_protocol::{R_B_START, UPDATE_SEASON_NOTICE};
use tools::templates::emoji_temp::EmojiTemp;
use tools::util::packet::Packet;

///行动请求
#[track_caller]
pub fn action(bm: &mut BattleMgr, packet: Packet) -> anyhow::Result<()> {
    let rm_ptr = bm as *mut BattleMgr;
    let user_id = packet.get_user_id();
    let res = bm.get_room_mut(&user_id);
    if let None = res {
        return Ok(());
    }
    let room = res.unwrap();
    //校验房间状态
    if room.get_state() != RoomState::BattleStarted {
        warn!(
            "room state is not battle_started!room_id:{},state:{:?}",
            room.get_room_id(),
            room.state
        );
        return Ok(());
    }

    //校验用户
    let res = check_user(room, user_id);
    if !res {
        return Ok(());
    }

    //解析protobuf
    let mut ca = C_ACTION::new();
    let res = ca.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
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
            return Ok(());
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
            return Ok(());
        }
    }

    //如果有问题就返回
    if let Err(_) = res {
        return Ok(());
    }

    //回给客户端,添加action主动方
    let mut san = S_ACTION_NOTICE::new();
    san.action_uints.push(au);

    //添加额外的行动方
    if let Ok(au_vec) = res {
        if let Some(v) = au_vec {
            for au in v {
                san.action_uints.push(au);
            }
        }
    }

    //以下通知客户端结果
    let bytes = san.write_to_bytes();
    if let Err(e) = bytes {
        error!("{:?}", e);
        return Ok(());
    }
    let bytes = bytes.unwrap();
    //处理只用推送给自己的技能
    if (action_type == ActionType::Skill || action_type == ActionType::EndShowMapCell)
        && PUSH_SELF.contains(&value)
    {
        room.send_2_client(ClientCode::ActionNotice, user_id, bytes.clone());
    } else {
        //推送给所有房间成员
        room.send_2_all_client(ClientCode::ActionNotice, bytes);
    }

    unsafe {
        let cter = room.battle_data.get_battle_cter(None, false).unwrap();
        let current_cter_is_died = cter.is_died();
        //判断是否进行结算
        let is_summary = process_summary(rm_ptr.as_mut().unwrap(), room);
        if !is_summary && current_cter_is_died {
            room.battle_data.next_turn();
        }
    }
    Ok(())
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
    let room = bm.rooms.remove(&room_id);
    if let None = room {
        return true;
    }
    let room = room.unwrap();

    for user_id in room.battle_data.battle_cter.keys() {
        bm.player_room.remove(user_id);
    }
    true
}

///开始战斗
pub fn start(bm: &mut BattleMgr, packet: Packet) -> anyhow::Result<()> {
    let mut rbs = R_B_START::new();
    let res = rbs.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        warn!("{:?}", e);
        return Ok(());
    }

    let rt = rbs.get_room_pt();
    let tcp_sender = bm.get_game_center_channel_clone();
    let task_sender = bm.get_task_sender_clone();
    let robot_task_sender = bm.get_robot_task_sender_clone();
    //创建战斗房间
    let room = Room::new(rt, tcp_sender, task_sender, robot_task_sender);
    if let Err(e) = room {
        error!("create battle room fail!{:?}", e);
        return Ok(());
    }
    let mut room = room.unwrap();
    //开始战斗
    room.start();
    let room_id = room.get_room_id();
    for user_id in room.members.keys() {
        bm.player_room.insert(*user_id, room_id);
    }
    bm.rooms.insert(room.get_room_id(), room);

    Ok(())
}

///处理pos
pub fn pos(rm: &mut BattleMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);
    if let None = res {
        return Ok(());
    }
    let room = res.unwrap();
    //校验房间状态
    if room.get_state() != RoomState::BattleStarted {
        warn!(
            "room state is not battle_started!room_id:{}",
            room.get_room_id()
        );
        return Ok(());
    }

    let battle_cter = room.get_battle_cter_mut_ref(&user_id);
    if battle_cter.is_none() {
        warn!("battle_cter is not find!user_id:{}", user_id);
        return Ok(());
    }
    let battle_cter = battle_cter.unwrap();

    let mut cp = C_POS::new();
    let res = cp.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let skill_id = cp.skill_id;
    let pos_type = cp.field_type;
    //校验技能
    let res = battle_cter.skills.contains_key(&skill_id);
    if !res {
        warn!(
            "this battle_cter has no this skill!cter_id:{},skill_id:{}",
            battle_cter.get_cter_id(),
            skill_id
        );
        return Ok(());
    }
    //校验操作类型
    if pos_type < PosType::ChangePos as u32 || pos_type > PosType::CancelPos as u32 {
        warn!(
            "the pos_type is error!user_id:{},pos_type:{}",
            user_id, pos_type
        );
        return Ok(());
    }
    let mut spn = S_POS_NOTICE::new();
    spn.set_user_id(user_id);
    spn.set_field_type(pos_type);
    spn.set_skill_id(skill_id);
    let bytes = spn.write_to_bytes().unwrap();
    room.send_2_all_client(ClientCode::PosNotice, bytes);
    Ok(())
}

///使用道具
fn use_item(
    rm: &mut Room,
    user_id: u32,
    item_id: u32,
    targets: Vec<u32>,
    au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
    let battle_cter = rm.battle_data.battle_cter.get(&user_id);
    if let None = battle_cter {
        let str = format!("battle_cter is not find!user_id:{}", user_id);
        warn!("{:?}", str);
        anyhow::bail!(str)
    }
    let battle_cter = battle_cter.unwrap();

    if battle_cter.is_locked() {
        let str = format!("battle_cter is locked!user_id:{}", user_id);
        warn!("{:?}", str);
        anyhow::bail!(str)
    }

    if battle_cter.items.is_empty() {
        let str = format!("battle_cter is not find!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    let res = battle_cter.items.contains_key(&item_id);
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
) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
    let battle_data = rm.battle_data.borrow();
    let battle_cter = rm.battle_data.battle_cter.get(&user_id).unwrap();
    if battle_cter.is_locked() {
        let str = format!("battle_cter is locked!user_id:{}", user_id);
        warn!("{:?}", str);
        anyhow::bail!(str)
    }
    //校验地图块
    let res = battle_data.check_choice_index(target_map_cell_index, true, true, true, true, false);
    if let Err(e) = res {
        let str = format!("{:?}", e);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    //校验剩余翻块次数
    if battle_cter.flow_data.residue_open_times <= 0 {
        let str = format!("this player's residue_open_times is 0!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }

    //校验本turn是否翻过
    if battle_cter
        .flow_data
        .open_map_cell_vec
        .contains(&target_map_cell_index)
    {
        let str = format!(
            "this player already has open this map_cell!user_id:{},open_map_cell_vec:{:?},index:{}",
            user_id, battle_cter.flow_data.open_map_cell_vec, target_map_cell_index
        );
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
) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
    //先校验玩家是否可以进行攻击
    let battle_cter = rm.battle_data.battle_cter.get(&user_id).unwrap();
    if battle_cter.is_locked() {
        let str = format!("battle_cter is locked!user_id:{}", user_id);
        warn!("{:?}", str);
        anyhow::bail!(str)
    }
    if !battle_cter.is_can_attack() {
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
) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
    //校验技能id有效性
    let battle_cter = rm.battle_data.battle_cter.get_mut(&user_id).unwrap();
    if battle_cter.is_locked() {
        let str = format!("battle_cter is locked!user_id:{}", user_id);
        warn!("{:?}", str);
        anyhow::bail!(str)
    }
    let skill = battle_cter.skills.get(&skill_id);
    if skill.is_none() {
        warn!("this skill is none!skill_id:{}", user_id);
        anyhow::bail!("")
    }
    //校验技能可用状态
    let skill = skill.unwrap();
    let res = check_skill_useable(battle_cter, skill);
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
) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
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
    let battle_cter = rm.battle_data.battle_cter.get(&user_id);
    if battle_cter.is_none() {
        warn!("skip_choice_turn battle_cter is none!user_id:{}", user_id);
        anyhow::bail!("")
    }

    //没有翻过地图块，则跳过
    let battle_cter = battle_cter.unwrap();
    if !battle_cter.get_is_can_end_turn() {
        warn!("this player not open any map_cell yet!user_id:{}", user_id);
        anyhow::bail!("")
    }
    //跳过当前这个人
    rm.battle_data.skip_turn(_au);
    rm.refresh_map();
    Ok(None)
}

///发送表情
pub fn emoji(bm: &mut BattleMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let res = bm.get_room_mut(&user_id);
    if res.is_none() {
        warn!("this player is not in the room!user_id:{}", user_id);
        return Ok(());
    }
    let room = res.unwrap();
    let cter = room.battle_data.battle_cter.get(&user_id);
    if let None = cter {
        warn!("can not find cter!user_id:{}", user_id);
        return Ok(());
    }
    let cter = cter.unwrap();
    let cter_id = cter.base_attr.cter_id;

    let mut ce = C_EMOJI::new();
    let res = ce.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let emoji_id = ce.emoji_id;
    let res: Option<&EmojiTemp> = crate::TEMPLATES
        .get_emoji_temp_mgr_ref()
        .temps
        .get(&emoji_id);
    if res.is_none() {
        warn!("there is no temp for emoji_id:{}", emoji_id);
        return Ok(());
    }
    //校验表情是否需要解锁和角色表情
    let emoji = res.unwrap();
    if emoji.condition != 0 {
        warn!("this emoji need unlock!emoji_id:{}", emoji_id);
        return Ok(());
    } else if emoji.condition == 0 && emoji.cter_id > 0 && emoji.cter_id != cter_id {
        warn!(
            "this character can not send this emoji!cter_id:{},emoji_id:{}",
            cter_id, emoji_id
        );
        return Ok(());
    }
    //走正常逻辑
    room.emoji(user_id, emoji_id);
    Ok(())
}

///离开房间
pub fn off_line(bm: &mut BattleMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    //校验用户不在战斗房间里
    let room = bm.get_room_mut(&user_id);
    if room.is_none() {
        //通知游戏服卸载玩家数据
        bm.send_2_server(GameCode::UnloadUser.into_u32(), user_id, Vec::new());
        return Ok(());
    }
    let room_id = room.unwrap().get_room_id();
    //处理玩家离开
    bm.handler_leave(room_id, user_id, false);
    //通知游戏服卸载玩家数据
    bm.send_2_server(GameCode::UnloadUser.into_u32(), user_id, Vec::new());
    Ok(())
}

///离开房间
pub fn leave_room(bm: &mut BattleMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();

    //校验用户不在战斗房间里
    let room = bm.get_room_mut(&user_id);
    if room.is_none() {
        return Ok(());
    }
    let room_id = room.unwrap().get_room_id();
    //处理玩家离开
    bm.handler_leave(room_id, user_id, true);
    Ok(())
}

pub fn reload_temps(_: &mut BattleMgr, _: Packet) -> anyhow::Result<()> {
    let path = std::env::current_dir();
    if let Err(e) = path {
        anyhow::bail!("{:?}", e)
    }
    let path = path.unwrap();
    let str = path.as_os_str().to_str();
    if let None = str {
        anyhow::bail!("reload_temps can not path to_str!")
    }
    let str = str.unwrap();
    let res = str.to_string() + "/template";
    crate::TEMPLATES.reload_temps(res.as_str())?;
    info!("reload_temps success!");
    Ok(())
}

///更新赛季
pub fn update_season(_: &mut BattleMgr, packet: Packet) -> anyhow::Result<()> {
    let mut usn = UPDATE_SEASON_NOTICE::new();
    let res = usn.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    unsafe {
        SEASON.season_id = usn.get_season_id();
        let str = usn.get_last_update_time();
        let last_update_time = chrono::NaiveDateTime::parse_from_str(str, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .timestamp() as u64;
        let str = usn.get_next_update_time();
        let next_update_time = chrono::NaiveDateTime::parse_from_str(str, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .timestamp() as u64;
        SEASON.last_update_time = last_update_time;
        SEASON.next_update_time = next_update_time;
    }
    Ok(())
}

///选择初始占位
pub fn choice_index(bm: &mut BattleMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut ccl = C_CHOOSE_INDEX::new();
    ccl.merge_from_bytes(packet.get_data())?;
    let index = ccl.index;

    let room = bm.get_room_mut(&user_id);
    if room.is_none() {
        warn!("this player is not in the room!user_id:{}", user_id);
        return Ok(());
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
        return Ok(());
    }

    let res = room
        .battle_data
        .check_choice_index(index as usize, false, false, true, false, false);
    //校验参数
    if let Err(e) = res {
        warn!("{:?}", e);
        return Ok(());
    }

    //校验他选过没有
    let member = room.get_battle_cter_ref(&user_id).unwrap();
    if member.map_cell_index_is_choiced() {
        warn!("this player is already choice index!user_id:{}", user_id);
        return Ok(());
    }
    room.choice_index(user_id, index);
    Ok(())
}

///结束操作
fn unlock_oper(
    rm: &mut Room,
    user_id: u32,
    au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
    //先校验玩家是否可以进行攻击
    let battle_cter = rm.battle_data.battle_cter.get_mut(&user_id).unwrap();
    let v = *au.action_value.get(0).unwrap();
    if battle_cter.status.locked_oper == 0 || battle_cter.status.locked_oper != v {
        anyhow::bail!("{there is no show cell skill activate!}")
    }
    battle_cter.status.locked_oper = 0;
    battle_cter.set_is_can_end_turn(true);
    Ok(None)
}

///校验玩家
fn check_user(rm: &Room, user_id: u32) -> bool {
    let cter = rm.get_battle_cter_ref(&user_id);
    if let None = cter {
        warn!(
            "user is not in the room!room_id:{},user_id:{}",
            rm.get_room_id(),
            user_id
        );
        return false;
    }
    let cter = cter.unwrap();
    //校验角色是否死亡
    if cter.is_died() {
        warn!(
            "this cter is already dead!room_id:{},user_id:{}",
            rm.get_room_id(),
            user_id
        );
        return false;
    }
    //校验是否选择了占位
    if cter.index_data.map_cell_index.is_none() {
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
fn check_skill_useable(cter: &BattleCharacter, skill: &Skill) -> anyhow::Result<()> {
    //校验cd
    if skill.skill_temp.consume_type != SkillConsumeType::Energy.into_u8() && skill.cd_times > 0 {
        anyhow::bail!(
            "this skill cd is not ready!cter_id:{},skill_id:{},cd:{}",
            cter.get_cter_id(),
            skill.id,
            skill.cd_times
        )
    } else if skill.skill_temp.consume_type == SkillConsumeType::Energy.into_u8()
        && cter.base_attr.energy < skill.skill_temp.consume_value
    {
        anyhow::bail!(
            "this cter's energy is not enough!cter_id:{},skill_id:{},energy:{},cost_energy:{}",
            cter.get_cter_id(),
            skill.id,
            cter.base_attr.energy,
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
