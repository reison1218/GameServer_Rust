use crate::entity::battle::{ActionType, SkillConsumeType, TURN_DEFAULT_OPEN_CELL_TIMES};
use crate::entity::character::{BattleCharacter, Skill};
use crate::entity::map_data::CellType;
use crate::entity::room::{Room, RoomState};
use crate::mgr::room_mgr::RoomMgr;
use log::{error, info, warn};
use protobuf::Message;
use std::borrow::{Borrow, BorrowMut};
use std::fmt::Debug;
use tools::protos::battle::C_ACTION;
use tools::util::packet::Packet;

///翻地图块
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
    let action_type = ca.get_action_type();
    let mut target = ca.target;
    let value = ca.value;
    //行为分支
    let action_type = ActionType::from(action_type);
    match action_type {
        ActionType::None => {
            warn!("action_type is 0!");
            return Ok(());
        }
        ActionType::UseItem => use_item(room, user_id, value),
        ActionType::Skip => {
            skip_choice_turn(room, user_id);
        }
        ActionType::Open => {
            open_cell(room, user_id, value as usize);
        }
        ActionType::Skill => {
            use_skill(room, user_id, value, target);
        }
        ActionType::Attack => {
            attack(room, user_id, target);
        }
    }
    Ok(())
}

///使用道具
fn use_item(rm: &mut Room, user_id: u32, item_id: u32) {
    let battle_cter = rm.battle_data.battle_cter.get(&user_id).unwrap();
    if battle_cter.items.is_empty() {
        return;
    }
    let res = battle_cter.items.contains_key(&item_id);
    if !res {
        return;
    }
    let mut targets = Vec::new();
    targets.push(user_id);
    rm.battle_data.use_skill(item_id, targets);
}

///翻地图块
fn open_cell(rm: &mut Room, user_id: u32, target_cell_index: usize) {
    let battle_data = rm.battle_data.borrow();
    let battle_cter = rm.battle_data.battle_cter.get(&user_id).unwrap();
    //校验是否轮到自己
    if !check_is_user_turn(rm, user_id) {
        return;
    }
    //校验这个地图块能不能翻
    let cell = battle_data.tile_map.map.get(target_cell_index);
    if cell.is_none() {
        return;
    }

    //校验剩余翻块次数
    if battle_cter.residue_open_times == 0 {
        return;
    }

    let cell = cell.unwrap();

    //校验地图块有效性
    if cell.id < CellType::Valid as u32 {
        return;
    }

    //世界块不让翻
    if cell.is_world {
        return;
    }

    //锁住不让翻
    if cell.check_is_locked() {
        return;
    }

    //如果地图块已经配对，不能翻
    if cell.pair_index.is_some() {
        return;
    }

    //校验地图块合法性
    let res = battle_data.check_choice_index(target_cell_index);
    if !res {
        return;
    }
    let battle_data = rm.battle_data.borrow_mut();
    battle_data.open_cell(target_cell_index);
}

///进行普通攻击
fn attack(rm: &mut Room, user_id: u32, target_array: Vec<u32>) {
    //先校验玩家是否可以进行攻击
    let battle_cter = rm.battle_data.battle_cter.get(&user_id).unwrap();
    if !battle_cter.is_can_attack {
        return;
    }
    //如果目标为空
    if target_array.is_empty() {
        return;
    }
    rm.battle_data.attack(user_id, target_array);
}

///使用技能
fn use_skill(rm: &mut Room, user_id: u32, skill_id: u32, target_array: Vec<u32>) {
    //校验技能id有效性
    let battle_cter = rm.battle_data.battle_cter.get(&user_id).unwrap();
    let skill = battle_cter.skills.get(skill_id as usize);
    if skill.is_none() {
        return;
    }
    //校验技能可用状态
    let skill = skill.unwrap();
    let res = check_skill_useable(battle_cter, skill);
    if !res {
        return;
    }
    //使用技能，走正常逻辑
    rm.battle_data.use_skill(skill_id, target_array);
}

///跳过选择回合顺序
fn skip_choice_turn(rm: &mut Room, user_id: u32) {
    //判断是否是轮到自己操作
    let res = check_is_user_turn(rm, user_id);
    if !res {
        return;
    }

    //校验现在能不能跳过
    let next_user = rm.get_turn_user(None);
    if let Err(e) = next_user {
        warn!("{:?}", e);
        return;
    }
    //如果不是下一个turn的，不让跳过
    let next_user = next_user.unwrap();
    if next_user != user_id {
        warn!(
            "skip_choice_turn next_user!=user_id! next_user:{},user_id:{}",
            next_user, user_id
        );
        return;
    }

    //拿到战斗角色
    let battle_cter = rm.battle_data.battle_cter.get(&user_id);
    if battle_cter.is_none() {
        warn!("skip_choice_turn battle_cter is none!user_id:{}", user_id);
        return;
    }

    //没有翻过地图块，则跳过
    let battle_cter = battle_cter.unwrap();
    if battle_cter.recently_open_cell_index < 0 {
        return;
    }

    //跳过当前这个人
    rm.battle_data.skip_turn();
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
    fn get(&self, key: usize) -> Option<&T>;

    fn get_mut(&mut self, key: usize) -> Option<&mut T>;
}

impl Find<Skill> for Vec<Skill> {
    fn get(&self, key: usize) -> Option<&Skill> {
        for value in self.iter() {
            if value.id != key as u32 {
                continue;
            }
            return Some(value);
        }
        None
    }

    fn get_mut(&mut self, key: usize) -> Option<&mut Skill> {
        for value in self.iter_mut() {
            if value.id != key as u32 {
                continue;
            }
            return Some(value);
        }
        None
    }
}
