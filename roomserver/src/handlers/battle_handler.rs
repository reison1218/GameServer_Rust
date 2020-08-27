use crate::battle::battle_enum::{ActionType, PosType, SkillConsumeType};
use crate::battle::battle_skill::Skill;
use crate::mgr::room_mgr::RoomMgr;
use crate::room::character::BattleCharacter;
use crate::room::room::{Room, RoomState};
use crate::room::room_model::{RoomModel, RoomType};
use log::{error, warn};
use protobuf::Message;
use std::borrow::{Borrow, BorrowMut};
use std::convert::TryFrom;
use std::fmt::Debug;
use tools::cmd_code::ClientCode;
use tools::protos::base::ActionUnitPt;
use tools::protos::battle::{C_ACTION, C_POS, S_ACTION_NOTICE, S_POS_NOTICE};
use tools::util::packet::Packet;

///行动请求
pub fn action(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let rm_ptr = rm as *mut RoomMgr;
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);
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
    //判断是否是轮到该玩家操作
    let res = check_is_user_turn(room, user_id);
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
            res = use_item(room, user_id, value, &mut au);
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
            res = open_cell(room, user_id, value as usize, &mut au);
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
    for member_id in room.members.clone().keys() {
        room.send_2_client(ClientCode::ActionNotice, *member_id, bytes.clone());
    }

    //判断当前这个角色死了没,死了就直接下一个
    let cter = room.battle_data.get_battle_cter(None);
    if let Ok(cter) = cter {
        if cter.is_died() {
            //判断是否进行结算
            unsafe {
                let rm_ptr = rm as *mut RoomMgr;
                let room = rm_ptr.as_mut().unwrap().get_room_mut(&user_id).unwrap();
                let is_summary = battle_summary(rm_ptr.as_mut().unwrap(), room);
                if !is_summary {
                    room.battle_data.next_turn();
                }
            }
        }
    }
    Ok(())
}

///处理战斗结算
pub unsafe fn battle_summary(rm: &mut RoomMgr, room: &mut Room) -> bool {
    let is_summary = room.battle_summary();
    let room_type = room.get_room_type();
    let battle_type = room.setting.battle_type;
    let room_id = room.get_room_id();
    //如果要结算,卸载数据
    if !is_summary {
        return false;
    }
    let v = room.get_member_vec();
    match room_type {
        RoomType::Match => {
            let res = rm.match_rooms.get_match_room_mut(battle_type);
            res.rm_room(&room_id);
        }
        RoomType::Custom => {
            rm.custom_room.rm_room(&room_id);
        }
        _ => {}
    }
    for user_id in v {
        rm.player_room.remove(&user_id);
    }
    true
}

///处理pos
pub fn pos(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
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
    //判断是否是轮到该玩家操作
    let res = check_is_user_turn(room, user_id);
    if !res {
        warn!("now is not this user turn!user_id:{}", user_id);
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
            battle_cter.cter_id, skill_id
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
    for member_id in room.members.clone().keys() {
        room.send_2_client(ClientCode::PosNotice, *member_id, bytes.clone());
    }
    Ok(())
}

///使用道具
fn use_item(
    rm: &mut Room,
    user_id: u32,
    item_id: u32,
    au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
    let battle_cter = rm.battle_data.battle_cter.get(&user_id);
    if let None = battle_cter {
        let str = format!("battle_cter is not find!user_id:{}", user_id);
        error!("{:?}", str);
        anyhow::bail!(str)
    }
    let battle_cter = battle_cter.unwrap();
    if battle_cter.items.is_empty() {
        let str = format!("battle_cter is not find!user_id:{}", user_id);
        error!("{:?}", str.as_str());
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
    let res = rm.battle_data.use_item(user_id, item_id, au);
    Ok(res)
}

///翻地图块
fn open_cell(
    rm: &mut Room,
    user_id: u32,
    target_cell_index: usize,
    au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
    //校验是否轮到自己
    if !check_is_user_turn(rm, user_id) {
        let str = format!("is not this player'turn now!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    let battle_data = rm.battle_data.borrow();
    let battle_cter = rm.battle_data.battle_cter.get(&user_id).unwrap();

    //校验地图块
    let res = battle_data.check_choice_index(target_cell_index, true, true, true, false);
    if let Err(e) = res {
        let str = format!("{:?}", e);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    //校验剩余翻块次数
    if battle_cter.residue_open_times <= 0 {
        let str = format!("this player's residue_open_times is 0!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }

    let battle_data = rm.battle_data.borrow_mut();
    let res = battle_data.open_cell(target_cell_index, au);
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
    if !battle_cter.is_can_attack {
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
        rm.battle_data.attack(user_id, target_array, au);
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
    let battle_cter = rm.battle_data.battle_cter.get(&user_id).unwrap();
    let skill = battle_cter.skills.get(&skill_id);
    if skill.is_none() {
        warn!("this skill is none!skill_id:{}", user_id);
        anyhow::bail!("")
    }
    //校验技能可用状态
    let skill = skill.unwrap();
    let res = check_skill_useable(battle_cter, skill);
    if !res {
        warn!(
            "skill useable check fail!user_id:{},skill_id:{}",
            user_id, skill_id
        );
        anyhow::bail!("")
    }
    //使用技能，走正常逻辑
    let res = rm
        .battle_data
        .use_skill(user_id, skill_id, target_array, au);
    Ok(res)
}

///跳过选择回合顺序
fn skip_turn(
    rmgr: &mut RoomMgr,
    user_id: u32,
    _au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
    let rm = rmgr.get_room_mut(&user_id).unwrap();
    //判断是否是轮到自己操作
    let res = check_is_user_turn(rm, user_id);
    if !res {
        warn!("is not your turn!user_id:{}", user_id);
        anyhow::bail!("")
    }

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
    if battle_cter.open_cell_vec.is_empty() {
        warn!("this player not open any cell yet!user_id:{}", user_id);
        anyhow::bail!("")
    }

    unsafe {
        let rm_ptr = rmgr as *mut RoomMgr;
        let room = rm_ptr.as_mut().unwrap().get_room_mut(&user_id).unwrap();
        //跳过当前这个人
        room.battle_data.skip_turn(_au);
        room.refresh_map();
    }
    Ok(None)
}

///校验现在是不是该玩家回合
fn check_is_user_turn(rm: &Room, user_id: u32) -> bool {
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
fn check_skill_useable(cter: &BattleCharacter, skill: &Skill) -> bool {
    //校验cd
    if skill.skill_temp.consume_type != SkillConsumeType::Energy as u8 && skill.cd_times > 0 {
        return false;
    } else if skill.skill_temp.consume_type == SkillConsumeType::Energy as u8
        && cter.energy < skill.skill_temp.consume_value as u32
    {
        return false;
    }
    true
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
