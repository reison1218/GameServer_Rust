use crate::battle::battle::BattleData;
use crate::battle::battle_buff::Buff;
use crate::battle::battle_enum::skill_type::HURT_SELF_ADD_BUFF;
use crate::battle::battle_enum::EffectType::AddSkill;
use crate::battle::battle_enum::{EffectType, TargetType};
use crate::room::character::BattleCharacter;
use crate::room::map_data::Cell;
use crate::TEMPLATES;
use log::{error, warn};
use std::collections::HashMap;
use tools::protos::base::{ActionUnitPt, TargetPt};
use tools::templates::skill_temp::SkillTemp;

#[derive(Clone, Debug)]
pub struct Skill {
    pub id: u32,
    pub skill_temp: &'static SkillTemp,
    pub cd_times: i8, //剩余cd,如果是消耗能量则无视这个值
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
        }
    }
}

///地图块换位置
pub fn change_index(
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

    //先删掉
    let mut source_cell = battle_data.tile_map.map.remove(source_index);
    let mut target_cell = battle_data.tile_map.map.remove(target_index);

    let source_cell_user = source_cell.user_id;

    let target_cell_user = target_cell.user_id;

    //替换下标
    source_cell.index = target_index;
    target_cell.index = source_index;

    //替换上面的玩家id
    source_cell.user_id = target_cell_user;
    target_cell.user_id = source_cell_user;

    battle_data
        .tile_map
        .map
        .insert(source_cell.index, source_cell);
    battle_data
        .tile_map
        .map
        .insert(target_cell.index, target_cell);

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
    let index = *target_array.get(0).unwrap() as usize;
    //校验index合法性
    let res = battle_data.check_choice_index(index, true, true, true, false);
    if let Err(e) = res {
        warn!("show_index {:?}", e);
        return None;
    }

    let cell = battle_data.tile_map.map.get(index).unwrap();
    let cell_id = cell.id;
    //todo 下发给客户端
    let mut target_pt = TargetPt::new();
    target_pt.target_value.push(cell_id);
    target_pt.target_value.push(cell.index as u32);
    au.targets.push(target_pt);
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
    let cter = battle_data.get_battle_cter_mut(Some(user_id));
    if let Err(e) = cter {
        warn!("{:?}", e);
        return None;
    }
    let cter = cter.unwrap();
    let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id).unwrap();
    //先计算单体的
    let buff_id = skill_temp.buff as u32;

    let target_type = TargetType::from(skill_temp.target);

    let mut target_pt = TargetPt::new();
    match target_type {
        TargetType::PlayerSelf => {
            cter.add_buff(buff_id);
            target_pt.target_value.push(cter.cell_index as u32);
            target_pt.add_buffs.push(buff_id);
        }
        TargetType::UnPairNullCell => {
            let index = *target_array.get(0).unwrap() as usize;
            let cell = battle_data.tile_map.map.get_mut(index);
            if cell.is_none() {
                warn!("cell not find!index:{}", index);
                return None;
            }
            let cell = cell.unwrap();
            if cell.is_world {
                warn!("world_cell can not be choice!index:{}", index);
                return None;
            }
            if cell.pair_index.is_some() {
                warn!("this cell is already paired!index:{}", index);
                return None;
            }
            let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id).unwrap();
            let mut buff = Buff::from(buff_temp);
            buff.user_id = user_id;
            cell.buffs.push(buff);
            target_pt.target_value.push(index as u32);
            target_pt.add_buffs.push(buff_id);
        }
        _ => {}
    }
    au.targets.push(target_pt);

    //处理其他的
    if HURT_SELF_ADD_BUFF.contains(&skill_id) {
        let target_pt = battle_data.deduct_hp(user_id, user_id, Some(skill_temp.par1 as i32), true);
        match target_pt {
            Ok(target_pt) => au.targets.push(target_pt),
            Err(e) => error!("{:?}", e),
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
        let str = format!("{:?}", e);
        warn!("{:?}", str);
        return None;
    }

    let map = &mut battle_data.tile_map.map as *mut Vec<Cell>;

    //校验目标下标的地图块
    let cell = map.as_mut().unwrap().get_mut(target_index).unwrap();

    let battle_cter = battle_data.get_battle_cter_mut(Some(user_id));
    if let Err(e) = battle_cter {
        error!("{:?}", e);
        return None;
    }
    let battle_cter = battle_cter.unwrap();
    let mut pair_cell: Option<&mut Cell> = None;
    //找到与之匹配的地图块自动配对
    for _cell in map.as_mut().unwrap().iter_mut() {
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
    battle_cter.is_can_attack = false;

    let mut target_pt = TargetPt::new();
    target_pt.target_value.push(cell.id);
    target_pt.target_value.push(target_index as u32);
    au.targets.push(target_pt.clone());
    target_pt.target_value.push(pair_cell.id);
    target_pt.target_value.push(pair_cell.index as u32);
    au.targets.push(target_pt.clone());

    //处理配对触发逻辑
    let res = battle_data.open_cell_trigger_buff(user_id, au, true);
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
    let target_cter = battle_data.get_battle_cter_mut_by_cell_index(target_user_index);
    if let Err(e) = target_cter {
        warn!("{:?}", e);
        return None;
    }
    let target_cter = target_cter.unwrap();
    let target_cter_index = target_cter.cell_index;
    let target_user = target_cter.user_id;
    //更新目标玩家的下标
    let last_index = target_cter_index;
    target_cter.cell_index = target_index;
    let target_cell = battle_data.tile_map.map.get_mut(target_index);
    if let None = target_cell {
        warn!("there is no cell!index:{}", target_index);
        return None;
    }
    let target_cell = target_cell.unwrap();
    target_cell.user_id = target_user;

    //重制之前地图块上的玩家id
    let last_cell = battle_data.tile_map.map.get_mut(last_index).unwrap();
    last_cell.user_id = 0;

    let mut target_pt = TargetPt::new();
    target_pt.target_value.push(target_index as u32);
    au.targets.push(target_pt);

    //处理移动后事件
    let v = battle_data.handler_cter_move(target_user, target_index);
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
    let cter_index = battle_cter.cell_index;
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
    let target_type = TargetType::from(skill.skill_temp.target);
    let v = battle_data.cal_scope(user_id, cter_index, target_type, None, Some(scope_temp));

    let mut add_hp = 0_u32;
    let mut need_rank = true;
    for target_user in v {
        //扣血
        let target_pt = battle_data.deduct_hp(
            user_id,
            target_user,
            Some(skill.skill_temp.par1 as i32),
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
    let target_pt = battle_data.add_hp(user_id, add_hp as i32);
    if let Some(target_pt) = target_pt {
        au.targets.push(target_pt);
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
    let battle_cter = battle_data.get_battle_cter(Some(user_id)).unwrap();
    let skill = battle_cter.skills.get(&skill_id).unwrap();
    let damage = skill.skill_temp.par1 as i32;
    let damage_deep = skill.skill_temp.par2 as i32;
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
    let target_type = TargetType::from(skill.skill_temp.target);

    //计算符合中心范围内的玩家
    let v = battle_data.cal_scope(
        user_id,
        center_index,
        target_type,
        Some(target_array),
        Some(scope_temp),
    );

    let mut need_rank = true;
    for target_user in v {
        let cter = battle_data.get_battle_cter_mut(Some(target_user)).unwrap();
        let damage_res;
        //判断是否中心位置
        if cter.cell_index == center_index as usize && damage_deep > 0 {
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
    let target_user = *target_array.get(0).unwrap();

    let target_cter = battle_data.get_battle_cter_mut(Some(target_user));
    if let Err(e) = target_cter {
        warn!("{:?}", e);
        return None;
    }
    let skill = TEMPLATES.get_skill_ref().get_temp(&skill_id).unwrap();
    let target_pt = battle_data.deduct_hp(user_id, target_user, Some(skill.par1 as i32), true);
    if let Err(e) = target_pt {
        error!("{:?}", e);
        return None;
    }
    au.targets.push(target_pt.unwrap());
    //替换技能
    if skill.par2 > 0 {
        let user_id = au.from_user;
        let cter = battle_data.battle_cter.get_mut(&user_id).unwrap();
        cter.skills.remove(&skill_id);

        let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id);
        match skill_temp {
            Ok(st) => {
                let mut target_pt = TargetPt::new();
                target_pt.target_value.push(user_id);
                target_pt.lost_buffs.push(skill_id);
                target_pt.effect_type = AddSkill as u32;
                target_pt.effect_value = st.id;
                let skill = Skill::from(st);
                cter.skills.insert(skill.id, skill);
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
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
    let battle_cter = battle_data.get_battle_cter_mut(Some(target_user));
    if let Err(e) = battle_cter {
        warn!("{:?}", e);
        return None;
    }

    let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id).unwrap();

    let battle_cter = battle_cter.unwrap();

    let mut target_pt = TargetPt::new();
    target_pt.target_value.push(battle_cter.cell_index as u32);
    target_pt.effect_type = EffectType::SubSkillCd as u32;
    target_pt.effect_value = skill_temp.par1;
    au.targets.push(target_pt);
    battle_cter
        .skills
        .values_mut()
        .for_each(|skill| skill.sub_cd(Some(skill_temp.par1 as i8)));
    None
}
