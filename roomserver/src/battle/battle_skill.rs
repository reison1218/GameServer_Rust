use crate::battle::battle::BattleData;
use crate::battle::battle_buff::Buff;
use crate::battle::battle_enum::skill_type::{
    HURT_SELF_ADD_BUFF, MOVE_TO_NULL_CELL_AND_TRANSFORM, SHOW_ALL_USERS_CELL,
    SHOW_SAME_ELMENT_CELL_ALL, SHOW_SAME_ELMENT_CELL_ALL_AND_CURE, SKILL_DAMAGE_NEAR_DEEP,
    SKILL_OPEN_NEAR_CELL,
};
use crate::battle::battle_enum::{AttackState, EffectType, ElementType, TargetType};
use crate::battle::battle_trigger::TriggerEvent;
use crate::room::character::BattleCharacter;
use crate::room::map_data::Cell;
use crate::TEMPLATES;
use log::{error, warn};
use rand::{thread_rng, Rng};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::TryFrom;
use tools::protos::base::{ActionUnitPt, EffectPt, TargetPt};
use tools::templates::skill_temp::SkillTemp;

#[derive(Clone, Debug)]
pub struct Skill {
    pub id: u32,
    pub skill_temp: &'static SkillTemp,
    pub cd_times: i8,    //剩余cd,如果是消耗能量则无视这个值
    pub is_active: bool, //是否激活
}
impl Skill {
    ///减去技能cd
    pub fn sub_cd(&mut self, value: Option<i8>) {
        if let Some(value) = value {
            self.cd_times -= value;
        } else {
            self.cd_times -= 1;
        }
        if self.cd_times < 0 {
            self.cd_times = 0;
        }
    }

    ///增加技能cd
    pub fn add_cd(&mut self, value: Option<i8>) {
        if let Some(value) = value {
            self.cd_times += value;
        } else {
            self.cd_times += 1;
        }
    }

    ///重制技能cd
    pub fn reset_cd(&mut self) {
        self.cd_times = self.skill_temp.cd as i8;
    }
}

impl From<&'static SkillTemp> for Skill {
    fn from(skill_temp: &'static SkillTemp) -> Self {
        Skill {
            id: skill_temp.id,
            cd_times: 0,
            skill_temp: skill_temp,
            is_active: false,
        }
    }
}

///地图块换位置
pub unsafe fn change_index(
    battle_data: &mut BattleData,
    user_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    if target_array.len() < 2 {
        warn!(
            "target_array size is error!skill_id:{},user_id:{}",
            skill_id, user_id
        );
        return None;
    }
    let source_index = target_array.get(0).unwrap();
    let target_index = target_array.get(1).unwrap();

    let source_index = *source_index as usize;
    let target_index = *target_index as usize;

    let map_size = battle_data.tile_map.map.len();
    //校验地图块
    if source_index > map_size || target_index > map_size {
        warn!(
            "index is error!source_index:{},target_index:{}",
            source_index, target_index
        );
        return None;
    }

    //校验原下标
    let res = battle_data.check_choice_index(source_index, true, true, true, false);
    if let Err(e) = res {
        warn!("{:?}", e);
        return None;
    }

    //校验目标下标
    let res = battle_data.check_choice_index(target_index, true, true, true, false);
    if let Err(e) = res {
        warn!("{:?}", e);
        return None;
    }

    let map_ptr = battle_data.tile_map.map.borrow_mut() as *mut [Cell; 30];
    let mut source_cell = map_ptr.as_ref().unwrap().get(source_index).unwrap().clone();
    let mut target_cell = map_ptr.as_ref().unwrap().get(target_index).unwrap().clone();

    let source_cell_user = source_cell.user_id;
    let target_cell_user = target_cell.user_id;

    let (source_cell_2d_x, source_cell_2d_y) = (source_cell.x, source_cell.y);
    let (target_cell_2d_x, target_cell_2d_y) = (target_cell.x, source_cell.y);

    //替换下标
    source_cell.index = target_index;
    source_cell.x = target_cell_2d_x;
    source_cell.y = target_cell_2d_y;
    target_cell.index = source_index;
    target_cell.x = source_cell_2d_x;
    target_cell.y = source_cell_2d_y;

    //替换上面的玩家id
    source_cell.user_id = target_cell_user;
    target_cell.user_id = source_cell_user;

    map_ptr.as_mut().unwrap()[target_index] = source_cell;
    map_ptr.as_mut().unwrap()[source_index] = target_cell;

    //通知客户端
    let mut target_pt = TargetPt::new();
    target_pt.target_value.push(source_index as u32);
    au.targets.push(target_pt);
    None
}

///展示地图块
pub fn show_index(
    battle_data: &mut BattleData,
    user_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    //展示地图块
    if target_array.is_empty() {
        warn!(
            "target_array is empty!skill_id:{},user_id:{}",
            skill_id, user_id
        );
        return None;
    }
    //向所有玩家随机展示一个地图块，优先生命元素
    if SHOW_ALL_USERS_CELL == skill_id {
        let mut v = Vec::new();
        let mut nature_index = None;
        for index in battle_data.tile_map.un_pair_map.clone().keys() {
            let index = *index;
            let res = battle_data.check_choice_index(index, false, false, true, false);
            if let Err(_) = res {
                continue;
            }
            let cell = battle_data.tile_map.map.get(index).unwrap();
            if cell.element == ElementType::Nature.into_u8() {
                nature_index = Some(cell.index);
                break;
            }
            v.push(index);
        }
        let index;
        if nature_index.is_some() {
            index = nature_index.unwrap();
        } else {
            let mut rand = rand::thread_rng();
            index = rand.gen_range(0, v.len());
            let res = v.get(index);
            if let None = res {
                warn!("there is no cell can show!");
                return None;
            }
        }

        let cell = battle_data.tile_map.map.get(index).unwrap();
        let cell_id = cell.id;
        let mut target_pt = TargetPt::new();
        target_pt.target_value.push(cell_id);
        target_pt.target_value.push(cell.index as u32);
        au.targets.push(target_pt);
    } else if SHOW_SAME_ELMENT_CELL_ALL == skill_id {
        let index = *target_array.get(0).unwrap() as usize;
        let cell = battle_data.tile_map.map.get(index).unwrap();
        let element = cell.element;
        for _cell in battle_data.tile_map.map.iter() {
            if _cell.index == cell.index {
                continue;
            }
            if _cell.is_world {
                continue;
            }
            if _cell.element != element {
                continue;
            }
            let mut target_pt = TargetPt::new();
            target_pt.target_value.push(_cell.id);
            target_pt.target_value.push(_cell.index as u32);
            au.targets.push(target_pt);
        }
    } else if SHOW_SAME_ELMENT_CELL_ALL_AND_CURE == skill_id {
        let index = *target_array.get(0).unwrap() as usize;
        let cell = battle_data.tile_map.map.get(index).unwrap();
        let element = cell.element;
        let cell_id = cell.id;
        let cell_index = cell.index;
        std::mem::drop(cell);
        let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id);
        if let Err(e) = skill_temp {
            warn!("{:?}", e);
            return None;
        }
        let skill_temp = skill_temp.unwrap();
        let cter = battle_data
            .get_battle_cter_mut(Some(user_id), true)
            .unwrap();
        if skill_temp.par1 as u8 == element {
            let mut target_pt = TargetPt::new();
            target_pt.target_value.push(cter.get_cell_index() as u32);
            let mut ep = EffectPt::new();
            ep.set_effect_type(EffectType::AddEnergy.into_u32());
            ep.set_effect_value(skill_temp.par2);
            target_pt.effects.push(ep);
            au.targets.push(target_pt);
            cter.energy += skill_temp.par2 as u8;
            if cter.energy > cter.max_energy {
                cter.energy = cter.max_energy;
            }
        }
        let mut target_pt = TargetPt::new();
        target_pt.target_value.push(cell_id);
        target_pt.target_value.push(cell_index as u32);
        au.targets.push(target_pt);
    } else {
        //展示地图块
        let index = *target_array.get(0).unwrap() as usize;
        //校验index合法性
        let res = battle_data.check_choice_index(index, true, true, true, false);
        if let Err(e) = res {
            warn!("show_index {:?}", e);
            return None;
        }

        let cell = battle_data.tile_map.map.get(index).unwrap();
        let cell_id = cell.id;
        let mut target_pt = TargetPt::new();
        target_pt.target_value.push(cell_id);
        target_pt.target_value.push(cell.index as u32);
        au.targets.push(target_pt);
    }
    None
}

///上buff,121, 211, 221, 311, 322, 20002
pub unsafe fn add_buff(
    battle_data: &mut BattleData,
    user_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    let turn_index = battle_data.next_turn_index;

    let cter = battle_data.get_battle_cter_mut(Some(user_id), true);
    if let Err(e) = cter {
        warn!("{:?}", e);
        return None;
    }
    let cter = cter.unwrap();
    let cter = cter as *mut BattleCharacter;
    let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id).unwrap();
    //先计算单体的
    let buff_id = skill_temp.buff as u32;

    let target_type = TargetType::try_from(skill_temp.target as u8).unwrap();

    let mut target_pt = TargetPt::new();
    match target_type {
        TargetType::PlayerSelf => {
            cter.as_mut().unwrap().add_buff(
                Some(user_id),
                Some(skill_id),
                buff_id,
                Some(turn_index),
            );
            target_pt
                .target_value
                .push(cter.as_mut().unwrap().get_cell_index() as u32);
            target_pt.add_buffs.push(buff_id);
        }
        TargetType::UnPairNullCell => {
            let index = *target_array.get(0).unwrap() as usize;

            let res = battle_data.check_choice_index(index, true, true, false, true);
            if let Err(e) = res {
                warn!("{:?}", e);
                return None;
            }
            let cell = battle_data.tile_map.map.get_mut(index).unwrap();
            let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id).unwrap();
            let buff = Buff::new(
                buff_temp,
                Some(battle_data.next_turn_index),
                Some(user_id),
                Some(skill_id),
            );
            cell.buffs.insert(buff.id, buff);
            target_pt.target_value.push(index as u32);
            target_pt.add_buffs.push(buff_id);
        }
        _ => {}
    }
    //处理技能激活状态
    let skill = cter.as_mut().unwrap().skills.get_mut(&skill_id);
    if let Some(skill) = skill {
        skill.is_active = true;
    }
    au.targets.push(target_pt);

    //处理其他的
    if HURT_SELF_ADD_BUFF.contains(&skill_id) {
        let target_pt = battle_data.deduct_hp(user_id, user_id, Some(skill_temp.par1 as i16), true);
        match target_pt {
            Ok(target_pt) => au.targets.push(target_pt),
            Err(e) => error!("{:?}", e),
        }
    }
    None
}

///对翻开指定元素地图块上对玩家造成技能伤害
pub unsafe fn skill_damage_opened_element(
    battle_data: &mut BattleData,
    user_id: u32,
    skill_id: u32,
    _: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    let battle_data = battle_data as *mut BattleData;
    let cter = battle_data
        .as_mut()
        .unwrap()
        .get_battle_cter_mut(Some(user_id), true)
        .unwrap();
    let skill = cter.skills.get(&skill_id);
    if skill.is_none() {
        warn!(
            "can not find cter's skill!cter_id:{} skill_id:{}",
            cter.cter_id, skill_id
        );
        return None;
    }
    let skill = skill.unwrap();
    let skill_damage = skill.skill_temp.par1 as i16;
    let mut need_rank = true;
    for cell in battle_data.as_mut().unwrap().tile_map.map.iter() {
        if cell.element != skill.skill_temp.par2 as u8 {
            continue;
        }
        let target_user = cell.user_id;
        let target_cter = battle_data
            .as_mut()
            .unwrap()
            .get_battle_cter_mut(Some(target_user), true);
        if let Err(e) = target_cter {
            error!("{:?}", e);
            continue;
        }
        let target_pt = battle_data.as_mut().unwrap().deduct_hp(
            user_id,
            target_user,
            Some(skill_damage),
            need_rank,
        );
        if let Err(e) = target_pt {
            warn!("{:?}", e);
            continue;
        }
        let target_pt = target_pt.unwrap();
        au.targets.push(target_pt);
        need_rank = false;
    }
    need_rank = false;
    let tile_map = battle_data.as_mut().unwrap().tile_map.borrow_mut();
    for index in cter.open_cell_vec.iter() {
        let index = *index;
        let cell = tile_map.map.get(index).unwrap();
        if cell.element != skill.skill_temp.par2 as u8 {
            continue;
        }
        if cell.user_id != user_id {
            continue;
        }
        let target_pt = battle_data.as_mut().unwrap().deduct_hp(
            user_id,
            user_id,
            Some(skill_damage),
            need_rank,
        );
        if let Err(e) = target_pt {
            warn!("{:?}", e);
            continue;
        }
        let target_pt = target_pt.unwrap();
        au.targets.push(target_pt);
    }

    None
}

///使用技能翻地图块
pub unsafe fn skill_open_cell(
    battle_data: &mut BattleData,
    user_id: u32,
    skill_id: u32,
    _: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    if SKILL_OPEN_NEAR_CELL == skill_id {
        let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id);
        if let Err(e) = skill_temp {
            error!("{:?}", e);
            return None;
        }
        let skill_temp = skill_temp.unwrap();
        let scope_temp = TEMPLATES.get_skill_scope_ref().get_temp(&skill_temp.scope);
        if let Err(e) = scope_temp {
            error!("{:?}", e);
            return None;
        }
        let cter = battle_data.get_battle_cter(Some(user_id), true).unwrap();
        let scope_temp = scope_temp.unwrap();
        let (cells, _) = battle_data.cal_scope(
            user_id,
            cter.get_cell_index() as isize,
            TargetType::PlayerSelf,
            None,
            Some(scope_temp),
        );
        let mut v = Vec::new();
        for index in cells {
            let res = battle_data.check_choice_index(index, true, true, true, false);
            if let Err(_) = res {
                continue;
            }
            v.push(index);
        }
        let index = thread_rng().gen_range(0, v.len());
        let res = v.get(index);
        if let None = res {
            warn!("skill_open_cell!there is no cell can open!");
            return None;
        }

        let index = *res.unwrap();
        let cell = battle_data.tile_map.map.get(index).unwrap();
        let mut target_pt = TargetPt::new();
        target_pt.target_value.push(cell.id);
        target_pt.target_value.push(index as u32);
        au.targets.push(target_pt);
        //处理配对触发逻辑
        let res = battle_data.open_cell_buff_trigger(user_id, au, false);
        if let Err(e) = res {
            warn!("{:?}", e);
            return None;
        }
        let res = res.unwrap();
        if let Some(res) = res {
            return Some(vec![res]);
        }
    }
    None
}

///自动配对地图块
pub unsafe fn auto_pair_cell(
    battle_data: &mut BattleData,
    user_id: u32,
    _: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    //将1个地图块自动配对。本回合内不能攻击。
    let target_index = *target_array.get(0).unwrap() as usize;
    let res = battle_data.check_choice_index(target_index, true, true, true, false);
    if let Err(e) = res {
        warn!("{:?}", e);
        return None;
    }

    let map = &mut battle_data.tile_map.map as *mut [Cell; 30];

    //校验目标下标的地图块
    let cell = map.as_mut().unwrap().get_mut(target_index).unwrap();

    let battle_cter = battle_data.get_battle_cter_mut(Some(user_id), true);
    if let Err(e) = battle_cter {
        error!("{:?}", e);
        return None;
    }
    let battle_cter = battle_cter.unwrap();
    let mut pair_cell: Option<&mut Cell> = None;
    //找到与之匹配的地图块自动配对
    for _cell in map.as_mut().unwrap().iter_mut() {
        //排除自己
        if _cell.id == cell.id && _cell.index == cell.index {
            continue;
        }
        if _cell.id != cell.id {
            continue;
        }
        _cell.pair_index = Some(cell.index);
        cell.pair_index = Some(_cell.index);
        pair_cell = Some(_cell);
        break;
    }

    if pair_cell.is_none() {
        warn!(
            "there is no cell pair for target_index:{},target_cell_id:{}",
            target_index, cell.id
        );
        return None;
    }

    let pair_cell = pair_cell.unwrap();
    //处理本turn不能攻击
    battle_cter.attack_state = AttackState::Locked;

    let mut target_pt = TargetPt::new();
    target_pt.target_value.push(cell.id);
    target_pt.target_value.push(target_index as u32);
    au.targets.push(target_pt.clone());
    target_pt.target_value.clear();
    target_pt.target_value.push(pair_cell.id);
    target_pt.target_value.push(pair_cell.index as u32);
    au.targets.push(target_pt);

    //处理配对触发逻辑
    let res = battle_data.open_cell_buff_trigger(user_id, au, true);
    if let Err(e) = res {
        warn!("{:?}", e);
        return None;
    }
    let res = res.unwrap();
    if let Some(res) = res {
        return Some(vec![res]);
    }
    None
}

///移动玩家
pub unsafe fn move_user(
    battle_data: &mut BattleData,
    user_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    //选择一个玩家，将其移动到一个空地图块上。
    if target_array.len() < 2 {
        warn!(
            "move_user,the target_array size is error! skill_id:{},user_id:{}",
            skill_id, user_id
        );
        return None;
    }
    let target_user_index = *target_array.get(0).unwrap() as usize;
    let target_index = *target_array.get(1).unwrap() as usize;
    //校验下标的地图块
    let res = battle_data.check_choice_index(target_index, false, false, false, true);
    if let Err(e) = res {
        warn!("{:?}", e);
        return None;
    }
    let mut target_pt = TargetPt::new();
    target_pt.target_value.push(target_index as u32);
    au.targets.push(target_pt);
    let target_cter = battle_data.get_battle_cter_mut_by_cell_index(target_user_index);
    if let Err(e) = target_cter {
        warn!("{:?}", e);
        return None;
    }
    let target_cter = target_cter.unwrap();
    let target_user = target_cter.user_id;
    //处理移动后事件
    let v = battle_data.handler_cter_move(target_user, target_index, au);
    if let Err(e) = v {
        warn!("{:?}", e);
        return None;
    }
    let v = v.unwrap();

    Some(v)
}

///对相邻的所有玩家造成1点技能伤害，并回复等于造成伤害值的生命值。
pub unsafe fn skill_damage_and_cure(
    battle_data: &mut BattleData,
    user_id: u32,
    skill_id: u32,
    _: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    let battle_cters = &mut battle_data.battle_cter as *mut HashMap<u32, BattleCharacter>;
    let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
    let cter_index = battle_cter.get_cell_index();
    let skill = battle_cter.skills.get_mut(&skill_id).unwrap();
    let res = TEMPLATES
        .get_skill_scope_ref()
        .get_temp(&skill.skill_temp.scope);
    if let Err(e) = res {
        error!("{:?}", e);
        return None;
    }
    let scope_temp = res.unwrap();
    let cter_index = cter_index as isize;
    let target_type = TargetType::try_from(skill.skill_temp.target as u8).unwrap();
    let (_, v) = battle_data.cal_scope(user_id, cter_index, target_type, None, Some(scope_temp));

    let mut add_hp = 0_u32;
    let mut need_rank = true;
    for target_user in v {
        //扣血
        let target_pt = battle_data.deduct_hp(
            user_id,
            target_user,
            Some(skill.skill_temp.par1 as i16),
            need_rank,
        );
        match target_pt {
            Ok(target_pt) => {
                au.targets.push(target_pt);
                add_hp += skill.skill_temp.par1;
            }
            Err(e) => error!("{:?}", e),
        }
        need_rank = false;
    }

    //给自己加血
    let target_pt = battle_data.add_hp(Some(user_id), user_id, add_hp as i16, None);
    match target_pt {
        Ok(target_pt) => {
            au.targets.push(target_pt);
        }
        Err(e) => {
            warn!("{:?}", e);
        }
    }
    None
}

///技能aoe伤害
pub unsafe fn skill_aoe_damage(
    battle_data: &mut BattleData,
    user_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    let battle_cter = battle_data.get_battle_cter(Some(user_id), true).unwrap();
    let skill = battle_cter.skills.get(&skill_id).unwrap();
    let damage = skill.skill_temp.par1 as i16;
    let damage_deep = skill.skill_temp.par2 as i16;
    let scope_id = skill.skill_temp.scope;
    let scope_temp = TEMPLATES.get_skill_scope_ref().get_temp(&scope_id);
    if let Err(e) = scope_temp {
        error!("{:?}", e);
        return None;
    }
    let scope_temp = scope_temp.unwrap();

    //校验下标
    for index in target_array.iter() {
        let cell = battle_data.tile_map.map.get(*index as usize);
        if let None = cell {
            warn!("there is no cell!index:{}", index);
            return None;
        }
    }

    let center_index = *target_array.get(0).unwrap() as isize;
    let target_type = TargetType::try_from(skill.skill_temp.target as u8).unwrap();

    //计算符合中心范围内的玩家
    let (_, v) = battle_data.cal_scope(
        user_id,
        center_index,
        target_type,
        Some(target_array),
        Some(scope_temp),
    );

    let mut need_rank = true;
    for target_user in v {
        let cter = battle_data
            .get_battle_cter_mut(Some(target_user), true)
            .unwrap();
        let damage_res;
        //判断是否中心位置
        if cter.get_cell_index() == center_index as usize && damage_deep > 0 {
            damage_res = damage_deep;
        } else {
            damage_res = damage;
        }
        let target_pt = battle_data.deduct_hp(user_id, target_user, Some(damage_res), need_rank);
        match target_pt {
            Ok(target_pt) => {
                au.targets.push(target_pt);
                need_rank = false;
            }
            Err(e) => error!("{:?}", e),
        }
    }
    None
}

///单体技能伤害
pub unsafe fn single_skill_damage(
    battle_data: &mut BattleData,
    user_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    let target_index = *target_array.get(0).unwrap();
    let target_cter = battle_data.get_battle_cter_mut_by_cell_index(target_index as usize);
    if let Err(e) = target_cter {
        warn!("{:?}", e);
        return None;
    }
    let target_cter = target_cter.unwrap();
    let target_user = target_cter.user_id;
    if target_cter.is_died() {
        warn!("this target is died!user_id:{}", target_cter.user_id);
        return None;
    }
    let skill_damage;

    let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id).unwrap();
    //目标在附近伤害加深
    if skill_id == SKILL_DAMAGE_NEAR_DEEP {
        let (_, users) = battle_data.cal_scope(
            user_id,
            target_index as isize,
            TargetType::try_from(skill_temp.target).unwrap(),
            None,
            None,
        );
        if users.contains(&target_user) {
            skill_damage = skill_temp.par2 as i16;
        } else {
            skill_damage = skill_temp.par1 as i16;
        }
    } else {
        skill_damage = skill_temp.par1 as i16;
    }
    let target_pt = battle_data.deduct_hp(user_id, target_user, Some(skill_damage), true);
    if let Err(e) = target_pt {
        error!("{:?}", e);
        return None;
    }
    au.targets.push(target_pt.unwrap());
    None
}

///减技能cd
pub unsafe fn sub_cd(
    battle_data: &mut BattleData,
    _: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    let target_user = *target_array.get(0).unwrap();
    //目标的技能CD-2。
    let battle_cter = battle_data.get_battle_cter_mut(Some(target_user), true);
    if let Err(e) = battle_cter {
        warn!("{:?}", e);
        return None;
    }

    let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id).unwrap();

    let battle_cter = battle_cter.unwrap();

    let mut target_pt = TargetPt::new();
    target_pt
        .target_value
        .push(battle_cter.get_cell_index() as u32);
    let mut ep = EffectPt::new();
    ep.effect_type = EffectType::SubSkillCd as u32;
    ep.effect_value = skill_temp.par1;
    target_pt.effects.push(ep);
    au.targets.push(target_pt);
    battle_cter
        .skills
        .values_mut()
        .for_each(|skill| skill.sub_cd(Some(skill_temp.par1 as i8)));
    None
}

///范围治疗
pub unsafe fn scope_cure(
    battle_data: &mut BattleData,
    user_id: u32,
    skill_id: u32,
    _: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    let battle_data = battle_data.borrow_mut() as *mut BattleData;
    let cter = battle_data
        .as_mut()
        .unwrap()
        .get_battle_cter_mut(Some(user_id), true)
        .unwrap();
    let skill = cter.skills.get(&skill_id).unwrap();
    let self_cure = skill.skill_temp.par1 as i16;
    let other_cure = skill.skill_temp.par2 as i16;
    let scope_id = skill.skill_temp.scope;
    let target_type = TargetType::try_from(skill.skill_temp.target);
    if let Err(e) = target_type {
        warn!("{:?}", e);
        return None;
    }
    let target_type = target_type.unwrap();
    let scope_temp = TEMPLATES.get_skill_scope_ref().get_temp(&scope_id);
    if let Err(e) = scope_temp {
        warn!("{:?}", e);
        return None;
    }

    let target_pt = battle_data
        .as_mut()
        .unwrap()
        .add_hp(Some(user_id), user_id, self_cure, None);
    if let Err(e) = target_pt {
        error!("{:?}", e);
        return None;
    }
    let target_pt = target_pt.unwrap();
    au.targets.push(target_pt);

    let scope_temp = scope_temp.unwrap();
    let (_, users) = battle_data.as_mut().unwrap().cal_scope(
        user_id,
        cter.get_cell_index() as isize,
        target_type,
        None,
        Some(scope_temp),
    );
    for other_id in users {
        let target_pt =
            battle_data
                .as_mut()
                .unwrap()
                .add_hp(Some(user_id), other_id, other_cure, None);
        if let Err(e) = target_pt {
            warn!("{:?}", e);
            continue;
        }
        let target_pt = target_pt.unwrap();
        au.targets.push(target_pt);
    }
    None
}

///变身系列技能
pub unsafe fn transform(
    battle_data: &mut BattleData,
    user_id: u32,
    skill_id: u32,
    targets: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<ActionUnitPt>> {
    let battle_data = battle_data.borrow_mut() as *mut BattleData;
    let cter = battle_data
        .as_mut()
        .unwrap()
        .get_battle_cter_mut(Some(user_id), true)
        .unwrap();
    let skill = cter.skills.get(&skill_id).unwrap();
    let buff_id = skill.skill_temp.buff;
    let transform_cter_id = skill.skill_temp.par2;
    let target_type = TargetType::try_from(skill.skill_temp.target);
    if let Err(e) = target_type {
        warn!("{:?}", e);
        return None;
    }
    let target_type = target_type.unwrap();
    //处理移动到空地图块并变身技能
    if MOVE_TO_NULL_CELL_AND_TRANSFORM == skill_id {
        let index = targets.get(0);
        if let None = index {
            warn!("transform!targets is empty!");
            return None;
        }
        let index = *index.unwrap() as usize;
        //检查选择对位置
        let res = battle_data
            .as_ref()
            .unwrap()
            .check_choice_index(index, true, true, true, true);
        if let Err(e) = res {
            error!("{:?}", e);
            return None;
        }

        //计算范围
        let scope_id = skill.skill_temp.scope;
        let scope_temp = TEMPLATES.get_skill_scope_ref().get_temp(&scope_id);
        if let Err(e) = scope_temp {
            warn!("{:?}", e);
            return None;
        }
        let skill_damage = skill.skill_temp.par1 as i16;
        let scope_temp = scope_temp.unwrap();
        //对周围对人造成伤害
        let (_, other_users) = battle_data.as_ref().unwrap().cal_scope(
            user_id,
            cter.get_cell_index() as isize,
            target_type,
            None,
            Some(scope_temp),
        );
        let mut need_rank = true;
        for user in other_users {
            let target_pt = battle_data.as_mut().unwrap().deduct_hp(
                user_id,
                user,
                Some(skill_damage),
                need_rank,
            );
            if let Err(e) = target_pt {
                warn!("{:?}", e);
                continue;
            }
            au.targets.push(target_pt.unwrap());
            need_rank = false;
        }
    }
    //处理变身
    let res = cter.transform(user_id, transform_cter_id, buff_id);
    match res {
        Err(e) => {
            error!("{:?}", e);
            return None;
        }
        Ok(target_pt) => {
            au.targets.push(target_pt);
        }
    }
    None
}
