use crate::battle::battle_enum::{ActionType, SkillConsumeType};
use crate::battle::battle_skill::Skill;
use crate::mgr::room_mgr::RoomMgr;
use crate::room::character::BattleCharacter;
use crate::room::room::{Room, RoomState};
use log::{error, warn};
use protobuf::Message;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use tools::cmd_code::ClientCode;
use tools::protos::base::ActionUnitPt;
use tools::protos::battle::{C_ACTION, S_ACTION_NOTICE};
use tools::util::packet::Packet;

///行动请求
pub fn action(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);
    if let None = res {
        return Ok(());
    }
    let room = res.unwrap();
    let next_user = room.get_turn_user(None);
    if let Err(e) = next_user {
        error!("{:?}", e);
        return Ok(());
    }
    let next_user = next_user.unwrap();
    if next_user != user_id {
        return Ok(());
    }
    //校验房间状态
    if room.get_state() != &RoomState::BattleStarted {
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
    if res.is_err() {
        error!("{:?}", res.err().unwrap());
        return Ok(());
    }

    let mut au = ActionUnitPt::new();
    au.action_value.push(ca.value);
    au.from_user = user_id;
    let action_type = ca.get_action_type();
    let target = ca.target;
    let value = ca.value;
    //行为分支
    let action_type = ActionType::from(action_type);
    let res;
    match action_type {
        ActionType::None => {
            warn!("action_type is 0!");
            return Ok(());
        }
        ActionType::UseItem => {
            au.action_type = ActionType::UseItem as u32;
            res = use_item(room, user_id, value, &mut au);
        }
        ActionType::Skip => {
            au.action_type = ActionType::Skip as u32;
            res = skip_turn(room, user_id, &mut au);
        }
        ActionType::Open => {
            au.action_type = ActionType::Open as u32;
            res = open_cell(room, user_id, value as usize, &mut au);
        }
        ActionType::Skill => {
            au.action_type = ActionType::Skill as u32;
            res = use_skill(room, user_id, value, target, &mut au);
        }
        ActionType::Attack => {
            au.action_type = ActionType::Attack as u32;
            res = attack(room, user_id, target, &mut au);
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

    //回给客户端
    let mut san = S_ACTION_NOTICE::new();
    san.action_uints.push(au);

    if let Ok(au_vec) = res {
        if let Some(v) = au_vec {
            for au in v {
                san.action_uints.push(au);
            }
        }
    } else {
        return Ok(());
    }

    let bytes = san.write_to_bytes();
    if let Err(e) = bytes {
        error!("{:?}", e);
        return Ok(());
    }
    let bytes = bytes.unwrap();
    for member_id in room.members.clone().keys() {
        room.send_2_client(ClientCode::ActionNotice, *member_id, bytes.clone());
    }

    unsafe {
        let room = room as *mut Room;
        let battle_cter = room
            .as_ref()
            .unwrap()
            .battle_data
            .get_battle_cter(Some(user_id));

        match battle_cter {
            Ok(cter) => {
                //如果没有剩余翻块次数了，就下一个turn
                if cter.residue_open_times <= 0 && !cter.is_can_attack {
                    room.as_mut().unwrap().battle_data.next_turn();
                }
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
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
    let battle_cter = rm.battle_data.battle_cter.get(&user_id).unwrap();
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
    rm.battle_data.use_item(user_id, item_id, au)
}

///翻地图块
fn open_cell(
    rm: &mut Room,
    user_id: u32,
    target_cell_index: usize,
    au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
    let battle_data = rm.battle_data.borrow();
    let battle_cter = rm.battle_data.battle_cter.get(&user_id).unwrap();
    //校验是否轮到自己
    if !check_is_user_turn(rm, user_id) {
        let str = format!("is not this player'turn now!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    //校验这个地图块能不能翻
    let cell = battle_data.tile_map.map.get(target_cell_index);
    if cell.is_none() {
        let str = format!("can not find this cell!index:{}", target_cell_index);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }

    //校验剩余翻块次数
    if battle_cter.residue_open_times <= 0 {
        let str = format!("this player's residue_open_times is 0!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }
    let cell = cell.unwrap();
    //校验地图块有效性
    let res = battle_data.check_open_cell(cell);
    if let Err(e) = res {
        warn!("{:?}", e);
        anyhow::bail!("")
    }

    let battle_data = rm.battle_data.borrow_mut();
    battle_data.open_cell(target_cell_index, au)
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
    unsafe { rm.battle_data.attack(user_id, target_array, au) }
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
    let skill = battle_cter.skills.get(skill_id as usize);
    if skill.is_none() {
        anyhow::bail!("this skill is none!skill_id:{}", user_id)
    }
    //校验技能可用状态
    let skill = skill.unwrap();
    let res = check_skill_useable(battle_cter, skill);
    if !res {
        anyhow::bail!("skill useable check fail!user_id:{}", skill_id)
    }
    //使用技能，走正常逻辑
    rm.battle_data
        .use_skill(user_id, skill_id, target_array, au)
}

///跳过选择回合顺序
fn skip_turn(
    rm: &mut Room,
    user_id: u32,
    _au: &mut ActionUnitPt,
) -> anyhow::Result<Option<Vec<ActionUnitPt>>> {
    //判断是否是轮到自己操作
    let res = check_is_user_turn(rm, user_id);
    if !res {
        let str = format!("is not your turn!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
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
        let str = format!(
            "skip_choice_turn next_user!=user_id! next_user:{},user_id:{}",
            next_user, user_id
        );
        warn!("{:?}", str.as_str());
        anyhow::bail!("{:?}", str)
    }

    //拿到战斗角色
    let battle_cter = rm.battle_data.battle_cter.get(&user_id);
    if battle_cter.is_none() {
        let str = format!("skip_choice_turn battle_cter is none!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!("{:?}", str)
    }

    //没有翻过地图块，则跳过
    let battle_cter = battle_cter.unwrap();
    if battle_cter.recently_open_cell_index.is_none() {
        let str = format!("this player not open any cell yet!user_id:{}", user_id);
        warn!("{:?}", str.as_str());
        anyhow::bail!(str)
    }

    //跳过当前这个人
    rm.battle_data.skip_turn(_au)
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
