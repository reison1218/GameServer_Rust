use crate::battle::battle::BattleData;
use crate::battle::battle_buff::Buff;
use crate::battle::battle_enum::buff_type::TRAPS;
use crate::battle::battle_enum::skill_type::{
    HURT_SELF_ADD_BUFF, SHOW_ALL_USERS_CELL, SHOW_INDEX_SAME_ELEMENT,
    SHOW_SAME_ELMENT_CELL_ALL_AND_CURE, SKILL_AOE_CENTER_DAMAGE_DEEP, SKILL_AOE_RED_SKILL_CD,
    SKILL_DAMAGE_NEAR_DEEP, SKILL_OPEN_NEAR_CELL, SUB_MAX_MOVE_POINT,
};
use crate::battle::battle_enum::{ActionType, EffectType, ElementType, TargetType};
use crate::battle::battle_helper::build_action_unit_pt;
use crate::battle::battle_trigger::TriggerEvent;
use crate::robot::robot_trigger::RobotTriggerType;
use crate::room::map_data::MapCell;
use crate::TEMPLATES;
use log::{error, warn};
use rand::Rng;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::TryFrom;
use tools::protos::base::{ActionUnitPt, EffectPt, TargetPt};
use tools::templates::skill_temp::SkillTemp;

use super::battle_cter::BattleCharacter;
use super::battle_enum::skill_type::SCOPE_CHANGE_SKILL_AOE;
use super::battle_enum::SkillConsumeType;

#[derive(Clone, Debug)]
pub struct Skill {
    pub id: u32,
    pub function_id: u32, //功能id
    pub skill_temp: &'static SkillTemp,
    pub cd_times: i8,    //剩余cd,如果是消耗能量则无视这个值
    pub is_active: bool, //是否激活
}
impl Skill {
    ///加减技能cd,
    pub fn add_cd(&mut self, value: i8) {
        self.cd_times += value;
        if self.cd_times < 0 {
            self.cd_times = 0;
        } else if self.cd_times > self.skill_temp.cd as i8 {
            self.cd_times = self.skill_temp.cd as i8;
        }
    }

    pub fn clean_cd(&mut self) {
        self.cd_times = 0;
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
            function_id: skill_temp.function_id,
            cd_times: 0,
            skill_temp,
            is_active: false,
        }
    }
}

///地图块换位置
pub unsafe fn change_map_cell_index(
    battle_data: &mut BattleData,
    cter_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    _: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    if target_array.len() < 2 {
        warn!(
            "target_array size is error!skill_id:{},cter_id:{}",
            skill_id, cter_id
        );
        return None;
    }
    let source_index = target_array.get(0).unwrap();
    let target_index = target_array.get(1).unwrap();

    let source_index = *source_index as usize;
    let target_index = *target_index as usize;

    let map_size = battle_data.tile_map.map_cells.len();
    //校验地图块
    if source_index > map_size || target_index > map_size {
        warn!(
            "index is error!source_index:{},target_index:{}",
            source_index, target_index
        );
        return None;
    }
    //校验原下标
    let res = battle_data.check_choice_index(source_index, false, true, true, true, false, true);
    if let Err(e) = res {
        warn!("{:?}", e);
        return None;
    }

    //校验目标下标
    let res = battle_data.check_choice_index(target_index, false, true, true, true, false, true);
    if let Err(e) = res {
        warn!("{:?}", e);
        return None;
    }

    let map_ptr = battle_data.tile_map.map_cells.borrow_mut() as *mut [MapCell; 30];
    //玩家proto封装
    let mut au_vec = vec![];
    //目标proto
    let mut target_pt;
    //效果proto
    let mut ep;
    //陷阱
    let mut traps = None;
    //视野可见目标
    let mut view_targets = HashMap::new();

    //源地图块数据
    let source_map_cell = map_ptr.as_mut().unwrap().get_mut(source_index).unwrap();
    let source_cter_id = source_map_cell.cter_id;
    let (s_x, s_y) = (source_map_cell.x, source_map_cell.y);
    let source_has_lock_buff = source_map_cell.has_lock_buff();
    let source_traps = source_map_cell.get_traps_mut();

    //目标地图看数据
    let target_map_cell = map_ptr.as_mut().unwrap().get_mut(target_index).unwrap();
    let target_cter_id = target_map_cell.cter_id;
    let (t_x, t_y) = (target_map_cell.x, target_map_cell.y);
    let target_has_lock_buff = target_map_cell.has_lock_buff();
    let target_traps = target_map_cell.get_traps_mut();

    //加载全地图可见的，此处只有锁buff
    let mut index = None;
    if source_has_lock_buff {
        index = Some((source_index as u32, target_index as u32));
    } else if target_has_lock_buff {
        index = Some((target_index as u32, source_index as u32));
    }

    //再加载部分可见的陷阱类buff
    if !source_traps.is_empty() {
        traps = Some((((source_index as u32, target_index as u32)), source_traps));
    } else if target_traps.is_empty() {
        traps = Some((((target_index as u32, source_index as u32)), target_traps));
    }

    //优先判断全地图可见的
    if index.is_some() {
        let (index1, index2) = index.unwrap();
        target_pt = TargetPt::new();
        target_pt.target_value.push(index1);
        ep = EffectPt::new();
        ep.effect_type = EffectType::ChangeCellIndex.into_u32();
        ep.effect_value = index2;
        target_pt.effects.push(ep);
        if !view_targets.contains_key(&0) {
            view_targets.insert(0, vec![]);
        }
        let res = view_targets.get_mut(&0).unwrap();
        res.push(target_pt);
    } else if let Some(traps) = traps {
        let ((index1, index2), traps) = traps;
        for buff in traps {
            for &view_user in buff.trap_view_users.iter() {
                if !view_targets.contains_key(&view_user) {
                    view_targets.insert(view_user, vec![]);
                }
                target_pt = TargetPt::new();
                target_pt.target_value.push(index1);
                ep = EffectPt::new();
                ep.effect_type = EffectType::ChangeCellIndex.into_u32();
                ep.effect_value = index2;
                target_pt.effects.push(ep);
                let res = view_targets.get_mut(&view_user).unwrap();
                res.push(target_pt);
            }
        }
    }

    let mut au_pt;
    for (from_user, target_pts) in view_targets {
        au_pt = build_action_unit_pt(0, ActionType::None, 0);
        for target_pt in target_pts {
            au_pt.targets.push(target_pt);
        }
        au_vec.push((from_user, au_pt));
    }

    //换内存数据
    std::mem::swap(source_map_cell, target_map_cell);
    source_map_cell.index = source_index;
    source_map_cell.x = s_x;
    source_map_cell.y = s_y;
    target_map_cell.index = target_index;
    target_map_cell.x = t_x;
    target_map_cell.x = t_y;
    source_map_cell.cter_id = source_cter_id;
    target_map_cell.cter_id = target_cter_id;

    //调用机器人触发器,这里走匹配地图块逻辑(删除记忆中的地图块)
    battle_data.map_cell_trigger_for_robot(source_index, RobotTriggerType::MapCellPair);
    Some(au_vec)
}

///展示地图块
pub fn show_index(
    battle_data: &mut BattleData,
    _: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    let skill_temp = TEMPLATES.skill_temp_mgr().get_temp(&skill_id).unwrap();
    let skill_function_id = skill_temp.function_id;
    let show_index;
    if SHOW_INDEX_SAME_ELEMENT == skill_function_id {
        let index = *target_array.get(0).unwrap() as usize;
        let res = battle_data.check_choice_index(index, true, false, false, true, false, false);
        //校验地图块
        if let Err(e) = res {
            warn!("{:?}", e);
            return None;
        }
        let map_cell = battle_data.tile_map.map_cells.get(index).unwrap();
        let battle_player = battle_data.get_battle_player(None, true);
        if let Err(e) = battle_player {
            warn!("{:?}", e);
            return None;
        }
        let battle_player = battle_player.unwrap();
        //地图块必须已翻开
        if !battle_player
            .flow_data
            .open_map_cell_vec_history
            .contains(&index)
            && map_cell.pair_index.is_none()
        {
            warn!(
                "this index is invalid!the map_cell must open!index:{}",
                index
            );
            return None;
        }
        let element = map_cell.element;
        for _map_cell in battle_data.tile_map.map_cells.iter() {
            let res = battle_data.check_choice_index(
                _map_cell.index,
                false,
                true,
                true,
                true,
                false,
                true,
            );
            if res.is_err() {
                continue;
            }

            if _map_cell.element != element {
                continue;
            }

            let mut target_pt = TargetPt::new();
            target_pt.target_value.push(_map_cell.index as u32);
            au.targets.push(target_pt);
        }
        show_index = index;
    } else {
        show_index = 0;
    }

    //调用触发器
    battle_data.map_cell_trigger_for_robot(show_index, RobotTriggerType::SeeMapCell);
    None
}

///展示地图块
pub fn show_map_cell(
    battle_data: &mut BattleData,
    cter_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    let temp = TEMPLATES.skill_temp_mgr().get_temp(&skill_id).unwrap();
    let skill_function_id = temp.function_id;
    if skill_function_id != SHOW_ALL_USERS_CELL && target_array.is_empty() {
        warn!(
            "target_array is empty!skill_id:{},cter_id:{}",
            skill_id, cter_id
        );
        return None;
    }
    let user_id = battle_data.get_user_id(cter_id).unwrap();

    let battle_data_ptr = battle_data as *mut BattleData;
    unsafe {
        let battle_data_mut = battle_data_ptr.as_mut().unwrap();
        let battle_data_ref = battle_data_ptr.as_ref().unwrap();
        let battle_player = battle_data_mut.get_battle_player_mut(None, true).unwrap();
        let battle_cter = battle_data.get_battle_cter_mut(cter_id, true);
        if let Err(e) = battle_cter {
            warn!("{:?}", e);
            return None;
        }

        let view_target_type = TargetType::try_from(temp.view_target);
        if let Err(e) = view_target_type {
            error!("{:?}", e);
            return None;
        }
        let mut au_vec = vec![];
        let view_target_type = view_target_type.unwrap();

        let battle_cter = battle_cter.unwrap();
        let show_index;
        let mut target_pt;

        //向所有玩家随机展示一个地图块，优先生命元素
        if SHOW_ALL_USERS_CELL == skill_function_id {
            let mut v = Vec::new();
            let mut nature_index = None;
            for index in battle_data.tile_map.un_pair_map.iter() {
                let (index, map_cell_id) = (*index.0, *index.1);
                //排除是自己当前turn翻了的
                if battle_player
                    .flow_data
                    .open_map_cell_vec_history
                    .contains(&index)
                {
                    continue;
                }
                let res =
                    battle_data.check_choice_index(index, false, true, false, false, false, false);
                if let Err(_) = res {
                    continue;
                }
                //放到列表里面
                v.push((index, map_cell_id));
                //判断是否是生命元素,如果是，则直接跳出循环
                let map_cell = battle_data.tile_map.map_cells.get(index).unwrap();
                if map_cell.element == ElementType::Nature.into_u8() {
                    nature_index = Some(v.len() - 1);
                    break;
                }
            }
            let index;
            if let Some(nature_index) = nature_index {
                index = nature_index;
            } else if !v.is_empty() {
                let mut rand = rand::thread_rng();
                index = rand.gen_range(0..v.len());
                let res = v.get(index);
                if let None = res {
                    warn!("there is no map_cell can show!");
                    return None;
                }
            } else {
                warn!("there is no nature_index and v_vec is empty!");
                return None;
            }

            let map_cell = v.get(index).unwrap();
            show_index = map_cell.0;
            let map_cell_id = map_cell.1;
            target_pt = TargetPt::new();
            target_pt.target_value.push(map_cell.0 as u32);
            target_pt.target_value.push(map_cell_id);
        } else if SHOW_SAME_ELMENT_CELL_ALL_AND_CURE == skill_function_id {
            let index = *target_array.get(0).unwrap() as usize;
            let map_cell = battle_data_ref.tile_map.map_cells.get(index).unwrap();
            let element = map_cell.element;
            let map_cell_id = map_cell.id;
            let map_cell_index = map_cell.index;
            let skill_temp = TEMPLATES.skill_temp_mgr().get_temp(&skill_id);
            if let Err(e) = skill_temp {
                warn!("{:?}", e);
                return None;
            }
            let skill_temp = skill_temp.unwrap();
            if skill_temp.par1 as u8 == element {
                let mut target_pt = TargetPt::new();
                target_pt
                    .target_value
                    .push(battle_cter.get_map_cell_index() as u32);
                let mut ep = EffectPt::new();
                ep.set_effect_type(EffectType::AddEnergy.into_u32());
                ep.set_effect_value(skill_temp.par2);
                target_pt.effects.push(ep);
                au.targets.push(target_pt);
                battle_cter.add_energy(skill_temp.par2 as i8);
            }

            target_pt = TargetPt::new();
            target_pt.target_value.push(map_cell_index as u32);
            target_pt.target_value.push(map_cell_id);
            show_index = map_cell_index;
        } else {
            //展示地图块
            let index = *target_array.get(0).unwrap() as usize;
            //校验index合法性
            let res = battle_data.check_choice_index(index, false, true, true, true, false, false);
            if let Err(e) = res {
                warn!("show_index {:?}", e);
                return None;
            }

            let map_cell = battle_data.tile_map.map_cells.get(index).unwrap();
            let map_cell_id = map_cell.id;
            show_index = map_cell.index;
            target_pt = TargetPt::new();
            target_pt.target_value.push(map_cell.index as u32);
            target_pt.target_value.push(map_cell_id);
        }

        //判断地图块有没有陷阱
        let map_cell = battle_data.tile_map.map_cells.get_mut(show_index);
        let mut au_trap_pt = build_action_unit_pt(0, ActionType::None, 0);
        if let Some(map_cell) = map_cell {
            let map_cell_index = map_cell.index;
            let traps = map_cell.get_traps_mut();
            for buff in traps {
                let buff_id = buff.get_id();
                let mut target_pt = TargetPt::new();
                target_pt.add_buffs.push(buff_id);
                target_pt.target_value.push(map_cell_index as u32);
                au_trap_pt.targets.push(target_pt);

                //处理陷阱可见玩家
                if view_target_type == TargetType::PlayerSelf {
                    buff.trap_view_users.insert(user_id);
                } else {
                    for &temp_user_id in battle_data.battle_player.keys() {
                        buff.trap_view_users.insert(temp_user_id);
                    }
                }
            }
        }
        //判断是不是只推送给自己
        if view_target_type == TargetType::PlayerSelf {
            let mut au_pt = build_action_unit_pt(cter_id, ActionType::Skill, skill_id);
            au_pt.targets.push(target_pt);

            au_vec.push((user_id, au_pt));
            if !au_trap_pt.targets.is_empty() {
                au_vec.push((user_id, au_trap_pt));
            }
        } else {
            au.targets.push(target_pt);
            if !au_trap_pt.targets.is_empty() {
                au_vec.push((0, au_trap_pt));
            }
        }

        battle_player.status.locked_oper = skill_id;
        battle_player.set_is_can_end_turn(false);
        //调用触发器
        battle_data.map_cell_trigger_for_robot(show_index, RobotTriggerType::SeeMapCell);
        Some(au_vec)
    }
}

///上buff,121, 211, 221, 311, 322, 20002
pub unsafe fn add_buff(
    battle_data: &mut BattleData,
    cter_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    let user_id = battle_data.get_user_id(cter_id).unwrap();
    let battle_data_ptr = battle_data as *mut BattleData;
    let battle_data_mut = battle_data_ptr.as_mut().unwrap();
    let turn_index = battle_data.next_turn_index;

    let battle_cter = battle_data.get_battle_cter_mut(cter_id, true);
    if let Err(e) = battle_cter {
        warn!("{:?}", e);
        return None;
    }
    let battle_cter = battle_cter.unwrap();

    let cter_index = battle_cter.get_map_cell_index();
    let skill_temp = TEMPLATES.skill_temp_mgr().get_temp(&skill_id).unwrap();
    //先计算单体的
    let buff_id = skill_temp.buff as u32;
    let buff_function_res = crate::TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
    let buff_function_id;
    match buff_function_res {
        Ok(buff_temp) => buff_function_id = buff_temp.function_id,
        Err(_) => buff_function_id = 0,
    }

    let skill_function_id = skill_temp.function_id;

    let target_type = TargetType::try_from(skill_temp.target as u8).unwrap();
    let view_target_type = TargetType::try_from(skill_temp.view_target).unwrap();
    let mut target_pt = TargetPt::new();
    let mut au_vec = vec![];
    let mut index = 0;
    if target_type != TargetType::PlayerSelf {
        let res = target_array.get(0);
        if let None = res {
            warn!("the target_array is empty!skill_id:{}", skill_id);
            return None;
        }
        index = *res.unwrap() as usize;
    }
    match target_type {
        TargetType::PlayerSelf => {
            battle_cter.add_buff(Some(cter_id), Some(skill_id), buff_id, Some(turn_index));
            target_pt.target_value.push(cter_index as u32);
            target_pt.add_buffs.push(buff_id);
            au.targets.push(target_pt);
        }
        TargetType::AnyEnemyCter => {
            let target_cter = battle_data_ptr
                .as_mut()
                .unwrap()
                .get_battle_cter_mut_by_map_cell_index(index);
            match target_cter {
                Ok(target_cter) => {
                    target_cter.add_buff(Some(cter_id), Some(skill_id), buff_id, Some(turn_index));
                    target_pt.target_value.push(index as u32);
                    target_pt.add_buffs.push(buff_id);
                    au.targets.push(target_pt);
                }
                Err(e) => {
                    warn!("{:?}", e);
                    return None;
                }
            }
        }
        TargetType::MapCell
        | TargetType::UnOpenMapCell
        | TargetType::UnPairMapCell
        | TargetType::NullMapCell
        | TargetType::UnPairNullMapCell
        | TargetType::OpenedMapCell
        | TargetType::UnOpenMapCellAndUnLock
        | TargetType::UnLockNullMapCell => {
            let map_cell = battle_data_mut.tile_map.map_cells.get_mut(index).unwrap();
            let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id).unwrap();
            let mut buff = Buff::new(
                buff_temp,
                Some(battle_data_mut.next_turn_index),
                Some(cter_id),
                Some(skill_id),
            );
            target_pt.target_value.push(index as u32);
            target_pt.add_buffs.push(buff_id);
            //处理视野目标
            if view_target_type == TargetType::PlayerSelf {
                let mut au_pt = build_action_unit_pt(cter_id, ActionType::Skill, skill_id);
                au_pt.targets.push(target_pt);
                au_vec.push((user_id, au_pt));
                if TRAPS.contains(&buff_function_id) {
                    buff.trap_view_users.insert(user_id);
                }
            } else {
                au.targets.push(target_pt);
                if TRAPS.contains(&buff_function_id) {
                    for &user_id in battle_data_mut.battle_player.keys() {
                        buff.trap_view_users.insert(user_id);
                    }
                }
            }
            map_cell.buffs.insert(buff.get_id(), buff);
        }
        _ => {}
    }

    //处理技能激活状态
    let skill = battle_cter.skills.get_mut(&skill_id);
    if let Some(skill) = skill {
        skill.is_active = true;
    }

    //处理其他的全局要看到的，此处为自残扣血和｜｜除最大行动点数技能
    if HURT_SELF_ADD_BUFF.contains(&skill_function_id) || SUB_MAX_MOVE_POINT == skill_function_id {
        let target_cter_id;
        if SUB_MAX_MOVE_POINT == skill_function_id {
            let target_cter = battle_data_ptr
                .as_ref()
                .unwrap()
                .get_battle_cter_by_map_cell_index(index)
                .unwrap();
            target_cter_id = target_cter.get_cter_id();
        } else if HURT_SELF_ADD_BUFF.contains(&skill_function_id) {
            target_cter_id = cter_id;
        } else {
            target_cter_id = 0;
        }
        let mut target_pt = battle_data.new_target_pt(target_cter_id).unwrap();
        let res = battle_data.deduct_hp(
            cter_id,
            target_cter_id,
            Some(skill_temp.par1 as i16),
            &mut target_pt,
            true,
        );
        match res {
            Ok(_) => {}
            Err(e) => error!("{:?}", e),
        }
        au.targets.push(target_pt);
    }

    Some(au_vec)
}

///对翻开指定元素地图块上对玩家造成技能伤害
pub fn skill_damage_opened_element(
    battle_data: &mut BattleData,
    cter_id: u32,
    skill_id: u32,
    _: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    let battle_cter = battle_data.get_battle_cter_mut(cter_id, true).unwrap();
    let skill = battle_cter.skills.get(&skill_id);
    if skill.is_none() {
        warn!(
            "can not find cter's skill!cter_id:{} skill_id:{}",
            battle_cter.get_cter_temp_id(),
            skill_id
        );
        return None;
    }
    let skill = skill.unwrap();
    let skill_damage = skill.skill_temp.par1 as i16;

    let target_array = battle_data.get_target_array(cter_id, skill_id);
    if let Err(e) = target_array {
        warn!("{:?}", e);
        return None;
    }
    let target_array = target_array.unwrap();
    let mut is_last_one = false;

    //计算技能伤害
    unsafe {
        for index_temp in 0..target_array.len() {
            let index = target_array.get(index_temp).unwrap();
            let index = *index;
            if index_temp == target_array.len() - 1 {
                is_last_one = true;
            }
            let map_cell = battle_data.tile_map.map_cells.get(index);
            if let None = map_cell {
                continue;
            }
            let target_cter_id = map_cell.unwrap().cter_id;
            let mut target_pt = battle_data.new_target_pt(target_cter_id).unwrap();
            let res = battle_data.deduct_hp(
                cter_id,
                target_cter_id,
                Some(skill_damage),
                &mut target_pt,
                is_last_one,
            );
            if let Err(e) = res {
                warn!("{:?}", e);
                continue;
            }
            au.targets.push(target_pt);
        }
    }
    None
}

///使用技能翻地图块
pub fn skill_open_map_cell(
    battle_data: &mut BattleData,
    cter_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    let skill = TEMPLATES.skill_temp_mgr().get_temp(&skill_id).unwrap();
    let skill_function_id = skill.function_id;
    if SKILL_OPEN_NEAR_CELL == skill_function_id {
        if target_array.is_empty() {
            warn!("{:?}", "target_array is empty");
            return None;
        }
        let index = *target_array.get(0).unwrap() as usize;
        let battle_cter = battle_data.get_battle_cter_mut(cter_id, true).unwrap();
        let robot_id = battle_cter.get_user_id();
        let cter_index = battle_cter.get_map_cell_index() as isize;
        let (map_cells, _) =
            battle_data.cal_scope(cter_id, cter_index, TargetType::PlayerSelf, None, None);

        //校验目标位置
        if !map_cells.contains(&index) {
            warn!("{:?}", "target_index is invalid!");
            return None;
        }

        //更新翻的地图块下标,使用技能翻格子不消耗翻块次数
        battle_data.exec_open_map_cell(robot_id, index);

        //处理配对逻辑
        let is_pair = battle_data.handler_map_cell_pair(robot_id, index);

        //封装target proto
        let map_cell = battle_data.tile_map.map_cells.get(index).unwrap();
        let mut target_pt = TargetPt::new();
        target_pt.target_value.push(index as u32);
        target_pt.target_value.push(map_cell.id);
        au.targets.push(target_pt);
        //处理配对触发逻辑
        let res = battle_data.open_map_cell_trigger(cter_id, au, is_pair);

        match res {
            Ok(res) => {
                if let Some(res) = res {
                    return Some(vec![res]);
                }
            }
            Err(e) => {
                warn!("{:?}", e);
                return None;
            }
        }
    }
    None
}

///自动配对地图块
pub unsafe fn auto_pair_map_cell(
    battle_data: &mut BattleData,
    cter_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    //将1个地图块自动配对。本回合内不能攻击。
    let target_index = *target_array.get(0).unwrap() as usize;
    let next_turn_index = battle_data.next_turn_index;
    let res = battle_data.check_choice_index(target_index, false, false, true, true, false, false);
    if let Err(e) = res {
        warn!("{:?}", e);
        return None;
    }
    let skill_temp = TEMPLATES.skill_temp_mgr().get_temp(&skill_id);
    if let Err(e) = skill_temp {
        warn!("{:?}", e);
        return None;
    }
    let skill_temp = skill_temp.unwrap();
    let buff_id = skill_temp.buff;

    let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
    if let Err(e) = buff_temp {
        warn!("{:?}", e);
        return None;
    }

    let battle_data_ptr = battle_data as *mut BattleData;
    let battle_data_mut = battle_data_ptr.as_mut().unwrap();
    let map = &mut battle_data.tile_map.map_cells as *mut [MapCell; 30];

    //校验目标下标的地图块
    let map_cell = battle_data_mut
        .tile_map
        .map_cells
        .get_mut(target_index)
        .unwrap();

    let battle_cter = battle_data.get_battle_cter_mut(cter_id, true);
    if let Err(e) = battle_cter {
        error!("{:?}", e);
        return None;
    }
    let battle_cter = battle_cter.unwrap();
    let user_id = battle_cter.base_attr.user_id;
    let battle_player = battle_data_ptr
        .as_mut()
        .unwrap()
        .get_battle_player_mut(Some(user_id), true)
        .unwrap();

    let mut pair_map_cell: Option<&mut MapCell> = None;
    let map_cell_index = map_cell.index;
    let mut map_cell_target_index = 0;
    //找到与之匹配的地图块自动配对
    for _map_cell in map.as_mut().unwrap().iter_mut() {
        //排除自己
        if _map_cell.id == map_cell.id && _map_cell.index == map_cell.index {
            continue;
        }
        if _map_cell.id != map_cell.id {
            continue;
        }
        map_cell_target_index = _map_cell.index;
        _map_cell.pair_index = Some(map_cell_index);
        map_cell.pair_index = Some(map_cell_target_index);
        //设置打开的人
        _map_cell.open_cter = cter_id;
        map_cell.open_cter = cter_id;
        //设置匹配的块
        pair_map_cell = Some(_map_cell);
        break;
    }

    if pair_map_cell.is_none() {
        warn!(
            "there is no map_cell pair for target_index:{},target_map_cell_id:{}",
            target_index, map_cell.id
        );
        return None;
    }

    let pair_map_cell = pair_map_cell.unwrap();

    //设置配对状态
    battle_player.status.is_pair = true;
    //处理本turn不能攻击
    battle_player.change_attack_locked();
    battle_cter.add_buff(None, None, buff_id, Some(next_turn_index));
    let cter_map_cell_index = battle_cter.get_map_cell_index() as u32;
    //清除已配对的
    battle_data.tile_map.un_pair_map.remove(&map_cell_index);
    battle_data
        .tile_map
        .un_pair_map
        .remove(&map_cell_target_index);

    let mut target_pt = TargetPt::new();
    target_pt.target_value.push(target_index as u32);
    target_pt.target_value.push(map_cell.id);
    au.targets.push(target_pt.clone());
    target_pt.target_value.clear();
    target_pt.target_value.push(pair_map_cell.index as u32);
    target_pt.target_value.push(pair_map_cell.id);
    au.targets.push(target_pt);
    //添加buff
    let mut target_pt = TargetPt::new();
    target_pt.target_value.push(cter_map_cell_index);
    target_pt.add_buffs.push(buff_id);
    au.targets.push(target_pt);

    //处理配对触发逻辑
    let res = battle_data.open_map_cell_trigger(cter_id, au, true);
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
pub fn move_user(
    battle_data: &mut BattleData,
    cter_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    //选择一个玩家，将其移动到一个空地图块上。
    if target_array.len() < 2 {
        warn!(
            "move_user,the target_array size is error! skill_id:{},cter_id:{}",
            skill_id, cter_id
        );
        return None;
    }
    let target_user_index = *target_array.get(0).unwrap() as usize;
    let target_index = *target_array.get(1).unwrap() as usize;
    //校验下标的地图块
    let res = battle_data.check_choice_index(target_index, false, false, false, false, true, true);
    if let Err(e) = res {
        warn!("{:?}", e);
        return None;
    }

    let target_cter = battle_data.get_battle_cter_mut_by_map_cell_index(target_user_index);
    if let Err(e) = target_cter {
        warn!("{:?}", e);
        return None;
    }
    let target_cter = target_cter.unwrap();
    let target_cter_id = target_cter.get_cter_id();

    //处理移动后事件
    unsafe {
        let v = battle_data.handler_cter_move(target_cter_id, target_index, au);
        if let Err(e) = v {
            warn!("{:?}", e);
            return None;
        }
        let mut target_pt = TargetPt::new();
        target_pt.target_value.push(target_user_index as u32);
        target_pt.target_value.push(target_index as u32);
        au.targets.push(target_pt);
        let (_, v) = v.unwrap();
        Some(v)
    }
}

///对相邻的所有玩家造成1点技能伤害，并回复等于造成伤害值的生命值。
pub unsafe fn skill_damage_and_cure(
    battle_data: &mut BattleData,
    cter_id: u32,
    skill_id: u32,
    _: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    let battle_data_ptr = battle_data as *mut BattleData;
    let battle_data_mut = battle_data_ptr.as_mut().unwrap();

    let battle_cter = battle_data.get_battle_cter_mut(cter_id, true);
    if let Err(e) = battle_cter {
        error!("{:?}", e);
        return None;
    }
    let battle_cter = battle_cter.unwrap();
    let cter_index = battle_cter.get_map_cell_index() as isize;
    let skill = battle_cter.skills.get_mut(&skill_id).unwrap();
    let res = TEMPLATES
        .skill_scope_temp_mgr()
        .get_temp(&skill.skill_temp.scope);
    if let Err(e) = res {
        error!("{:?}", e);
        return None;
    }
    let scope_temp = res.unwrap();
    let target_type = TargetType::try_from(skill.skill_temp.target as u8).unwrap();
    let (_, v) =
        battle_data_mut.cal_scope(cter_id, cter_index, target_type, None, Some(scope_temp));

    let mut add_hp = 0_u32;
    let mut is_last_one = false;

    for index in 0..v.len() {
        let &target_cter_id = v.get(index).unwrap();
        if index == v.len() - 1 {
            is_last_one = true;
        }
        let mut target_pt = battle_data_mut.new_target_pt(target_cter_id).unwrap();
        //扣血
        let res = battle_data_mut.deduct_hp(
            cter_id,
            target_cter_id,
            Some(skill.skill_temp.par1 as i16),
            &mut target_pt,
            is_last_one,
        );
        match res {
            Ok(_) => {
                au.targets.push(target_pt);
                add_hp += skill.skill_temp.par1;
            }
            Err(e) => error!("{:?}", e),
        }
    }

    //给自己加血
    let target_pt = battle_data.add_hp(Some(cter_id), cter_id, add_hp as u16, None);
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
    cter_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    let battle_cter = battle_data.get_battle_cter(cter_id, true).unwrap();
    let self_index = battle_cter.get_map_cell_index();
    let skill = battle_cter.skills.get(&skill_id).unwrap();
    let skill_function_id = skill.function_id;
    let par1 = skill.skill_temp.par1 as i16;
    let par2 = skill.skill_temp.par2 as i16;
    let par3 = skill.skill_temp.par3 as i16;
    let par4 = skill.skill_temp.par3 as i16;
    let mut scope_id = skill.skill_temp.scope;
    let round = battle_data.round;
    if skill_function_id == SCOPE_CHANGE_SKILL_AOE {
        if round == 1 {
            scope_id = 1;
        } else if round == 2 {
            scope_id = par2 as u32;
        } else if round == 3 {
            scope_id = par3 as u32;
        } else if round >= 4 {
            scope_id = par4 as u32;
        }
    }

    let scope_temp = TEMPLATES.skill_scope_temp_mgr().get_temp(&scope_id);
    if let Err(e) = scope_temp {
        error!("{:?}", e);
        return None;
    }
    let scope_temp = scope_temp.unwrap();

    let center_index = *target_array.get(0).unwrap() as isize;
    let target_type = TargetType::try_from(skill.skill_temp.target as u8).unwrap();

    //计算符合中心范围内的玩家
    let (_, v) = battle_data.cal_scope(
        cter_id,
        center_index,
        target_type,
        Some(target_array),
        Some(scope_temp),
    );

    let mut is_last_one = false;
    let mut count = 0i16;

    for index in 0..v.len() {
        let &target_cter_id = v.get(index).unwrap();
        if index == v.len() - 1 {
            is_last_one = true;
        }
        let battle_cter = battle_data.get_battle_cter(cter_id, true).unwrap();
        let damage_res;
        //判断是否中心位置
        let cter_index = battle_cter.get_map_cell_index();
        if cter_index == center_index as usize && skill_function_id == SKILL_AOE_CENTER_DAMAGE_DEEP
        {
            damage_res = par2;
        } else {
            damage_res = par1;
        }
        let mut target_pt = battle_data.new_target_pt(target_cter_id).unwrap();
        let res = battle_data.deduct_hp(
            cter_id,
            target_cter_id,
            Some(damage_res),
            &mut target_pt,
            is_last_one,
        );
        match res {
            Ok(_) => {
                au.targets.push(target_pt);
                count += 1;
            }
            Err(e) => error!("{:?}", e),
        }
    }

    //如果技能是造成aoe并减cd
    if skill_function_id == SKILL_AOE_RED_SKILL_CD {
        //处理减cd逻辑,如果造成伤害人数大于参数
        if count >= par2 {
            let battle_cter = battle_data.get_battle_cter_mut(cter_id, true);
            match battle_cter {
                Ok(battle_cter) => {
                    let skill = battle_cter.skills.get_mut(&skill_id).unwrap();
                    skill.reset_cd();
                    let reduce_cd = -(par3 as i8);
                    skill.add_cd(reduce_cd);
                    let mut target_pt = TargetPt::new();
                    target_pt.target_value.push(self_index as u32);
                    let mut effect_pt = EffectPt::new();
                    effect_pt.effect_type = EffectType::SubSkillCd.into_u32();
                    effect_pt.effect_value = par3 as u32;
                    target_pt.effects.push(effect_pt);
                    au.targets.push(target_pt);
                }
                Err(e) => {
                    warn!("{:?}", e);
                }
            }
        }
    }
    None
}

///单体技能伤害
pub unsafe fn single_skill_damage(
    battle_data: &mut BattleData,
    cter_id: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    if target_array.is_empty() {
        warn!(
            "single_skill_damage-target_array is empty!skill_id:{}",
            skill_id
        );
        return None;
    }
    let target_index = *target_array.get(0).unwrap();
    let target_cter = battle_data.get_battle_cter_mut_by_map_cell_index(target_index as usize);
    if let Err(e) = target_cter {
        warn!("{:?}", e);
        return None;
    }
    let target_cter = target_cter.unwrap();
    let target_cter_id = target_cter.get_cter_id();
    let skill_damage;

    let skill_temp = TEMPLATES.skill_temp_mgr().get_temp(&skill_id).unwrap();
    let skill_function_id = skill_temp.function_id;
    //目标在附近伤害加深
    if skill_function_id == SKILL_DAMAGE_NEAR_DEEP {
        let (_, users) = battle_data.cal_scope(
            cter_id,
            target_index as isize,
            TargetType::try_from(skill_temp.target).unwrap(),
            None,
            None,
        );
        if users.contains(&target_cter_id) {
            skill_damage = skill_temp.par2 as i16;
        } else {
            skill_damage = skill_temp.par1 as i16;
        }
    } else {
        skill_damage = skill_temp.par1 as i16;
    }

    let mut target_pt = battle_data.new_target_pt(target_cter_id).unwrap();
    let res = battle_data.deduct_hp(
        cter_id,
        target_cter_id,
        Some(skill_damage),
        &mut target_pt,
        true,
    );
    if let Err(e) = res {
        error!("{:?}", e);
        return None;
    }

    au.targets.push(target_pt);
    None
}

///减技能cd
pub unsafe fn sub_cd(
    battle_data: &mut BattleData,
    _: u32,
    skill_id: u32,
    target_array: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    let target_cter_id = *target_array.get(0).unwrap();
    //目标的技能CD-2。
    let battle_cter = battle_data.get_battle_cter_mut(target_cter_id, true);
    if let Err(e) = battle_cter {
        warn!("{:?}", e);
        return None;
    }

    let skill_temp = TEMPLATES.skill_temp_mgr().get_temp(&skill_id).unwrap();

    let battle_cter = battle_cter.unwrap();
    let battle_player_index = battle_cter.get_map_cell_index();
    let mut target_pt = TargetPt::new();
    target_pt.target_value.push(battle_player_index as u32);
    let mut ep = EffectPt::new();
    ep.effect_type = EffectType::SubSkillCd as u32;
    ep.effect_value = skill_temp.par1;
    target_pt.effects.push(ep);
    au.targets.push(target_pt);
    battle_cter
        .skills
        .values_mut()
        .for_each(|skill| skill.add_cd(-(skill_temp.par1 as i8)));
    None
}

///范围治疗
pub unsafe fn scope_cure(
    battle_data: &mut BattleData,
    cter_id: u32,
    skill_id: u32,
    _: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    let battle_data = battle_data.borrow_mut() as *mut BattleData;
    let battle_cter = battle_data
        .as_mut()
        .unwrap()
        .get_battle_cter_mut(cter_id, true)
        .unwrap();
    let cter_index = battle_cter.get_map_cell_index();
    let skill = battle_cter.skills.get(&skill_id).unwrap();
    let self_cure = skill.skill_temp.par1 as i16;
    let other_cure = skill.skill_temp.par2 as i16;
    let scope_id = skill.skill_temp.scope;
    let target_type = TargetType::try_from(skill.skill_temp.target);
    if let Err(e) = target_type {
        warn!("{:?}", e);
        return None;
    }
    let target_type = target_type.unwrap();
    let scope_temp = TEMPLATES.skill_scope_temp_mgr().get_temp(&scope_id);
    if let Err(e) = scope_temp {
        warn!("{:?}", e);
        return None;
    }

    let scope_temp = scope_temp.unwrap();
    let (_, users) = battle_data.as_mut().unwrap().cal_scope(
        cter_id,
        cter_index as isize,
        target_type,
        None,
        Some(scope_temp),
    );
    for other_cter_id in users {
        let res;
        if other_cter_id == cter_id {
            res = self_cure;
        } else {
            res = other_cure;
        }
        let target_pt =
            battle_data
                .as_mut()
                .unwrap()
                .add_hp(Some(cter_id), other_cter_id, res as u16, None);
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
    cter_id: u32,
    skill_id: u32,
    targets: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    let battle_data = battle_data.borrow_mut() as *mut BattleData;
    let next_turn_index = battle_data.as_ref().unwrap().next_turn_index;

    let battle_cter = battle_data
        .as_mut()
        .unwrap()
        .get_battle_cter_mut(cter_id, true)
        .unwrap();
    //处理移动到空地图块并变身技能
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
        .check_choice_index(index, false, false, false, true, true, true);
    if let Err(e) = res {
        error!("{:?}", e);
        return None;
    }

    let skill = battle_cter.skills.get_mut(&skill_id).unwrap();
    let scope_id = skill.skill_temp.scope;
    let skill_damage = skill.skill_temp.par1 as i16;
    let consume_type = skill.skill_temp.consume_type;
    let consume_value = skill.skill_temp.consume_value;
    let buff_id = skill.skill_temp.buff;
    let transform_cter_temp_id = skill.skill_temp.par2;
    let target_type = TargetType::try_from(skill.skill_temp.target);
    if let Err(e) = target_type {
        warn!("{:?}", e);
        return None;
    }
    let target_type = target_type.unwrap();

    //处理技能消耗
    if consume_type != SkillConsumeType::Energy as u8 {
        skill.reset_cd();
    } else {
        let mut v = consume_value as i8;
        v = v * -1;
        battle_cter.add_energy(v);
    }
    //处理变身
    let res = battle_cter.transform(cter_id, transform_cter_temp_id, buff_id, next_turn_index);
    match res {
        Err(e) => {
            error!("{:?}", e);
            return None;
        }
        Ok(mut target_pt) => {
            target_pt.target_value.push(index as u32);
            au.targets.push(target_pt);
        }
    }

    //更新位置
    let v = battle_data
        .as_mut()
        .unwrap()
        .handler_cter_move(cter_id, index, au);

    if let Err(e) = v {
        warn!("{:?}", e.to_string());
        return None;
    }
    let (is_died, v) = v.unwrap();
    //判断玩家死了没
    if is_died {
        return Some(v);
    }

    //计算范围
    let scope_temp = TEMPLATES.skill_scope_temp_mgr().get_temp(&scope_id);
    if let Err(e) = scope_temp {
        warn!("{:?}", e);
        return None;
    }
    let scope_temp = scope_temp.unwrap();
    //对周围对人造成伤害
    let (_, other_users) = battle_data.as_ref().unwrap().cal_scope(
        cter_id,
        index as isize,
        target_type,
        None,
        Some(scope_temp),
    );
    let mut is_last_one = false;

    for index in 0..other_users.len() {
        let user = other_users.get(index).unwrap();
        let user = *user;
        //排除自己
        if user == cter_id {
            continue;
        }
        if index == other_users.len() - 1 {
            is_last_one = true;
        }
        let mut target_pt = battle_data.as_ref().unwrap().new_target_pt(user).unwrap();
        let res = battle_data.as_mut().unwrap().deduct_hp(
            cter_id,
            user,
            Some(skill_damage),
            &mut target_pt,
            is_last_one,
        );
        if let Err(e) = res {
            warn!("{:?}", e);
            continue;
        }
        au.targets.push(target_pt);
    }

    Some(v)
}

///召唤宠物
pub unsafe fn summon_minon(
    battle_data: &mut BattleData,
    cter_id: u32,
    skill_id: u32,
    targets: Vec<u32>,
    au: &mut ActionUnitPt,
) -> Option<Vec<(u32, ActionUnitPt)>> {
    let skill_temp = crate::TEMPLATES.skill_temp_mgr().get_temp(&skill_id);
    if let Err(e) = skill_temp {
        error!("{:?}", e);
        return None;
    }
    let turn_index = battle_data.next_turn_index;
    let battle_data_ptr = battle_data as *mut BattleData;
    let battle_data_mut = battle_data_ptr.as_mut().unwrap();
    let battle_player = battle_data
        .get_battle_player_mut_by_cter_id(cter_id, true)
        .unwrap();
    let skill_temp = skill_temp.unwrap();
    let cter_temp_id = skill_temp.par1;
    let team_id = battle_player.team_id;
    let from_user_id = battle_player.get_user_id();
    {
        let battle_cter = battle_data_mut.get_battle_cter_mut(cter_id, true).unwrap();
        battle_cter.skills.get_mut(&skill_id).unwrap().is_active = true;
    }
    //遍历选中的地图块下标
    for index in targets {
        let index = index as usize;
        let res = battle_data_mut.check_choice_index(index, false, false, false, true, true, true);
        if let Err(e) = res {
            error!("{:?}", e);
            return None;
        }
        //生成新的角色id
        let new_cter_id: u32 = battle_data_mut.generate_cter_id();
        //创建新角色
        let minon = BattleCharacter::init_for_minon(
            from_user_id,
            team_id,
            cter_id,
            skill_id,
            new_cter_id,
            cter_temp_id,
            index,
            turn_index,
        );
        if let Err(e) = minon {
            error!("{:?}", e);
            return None;
        }
        let minon = minon.unwrap();
        let battle_cter_pt = minon.convert_to_battle_cter_pt();
        //封装数据
        battle_data_mut
            .cter_player
            .insert(new_cter_id, from_user_id);
        battle_player.cters.insert(new_cter_id, minon);

        //封装客户的消息
        let mut target_pt = battle_data_mut.new_target_pt(new_cter_id).unwrap();
        target_pt.set_new_cter(battle_cter_pt);
        au.targets.push(target_pt);
    }

    None
}
