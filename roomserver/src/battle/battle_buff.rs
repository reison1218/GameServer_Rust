use crate::battle::battle::{BattleData, Direction, Item};
use crate::battle::battle_enum::buff_type::{
    AWARD_BUFF, AWARD_ITEM, NEAR_ADD_CD, NEAR_SKILL_DAMAGE, OPEN_CELL_AND_PAIR, PAIR_CURE,
    SAME_CELL_ELEMENT_ADD_ATTACK,
};
use crate::battle::battle_enum::{ActionType, BattleCterState, EffectType};
use crate::battle::battle_enum::{TargetType, TRIGGER_SCOPE_NEAR};
use crate::handlers::battle_handler::{Delete, Find};
use crate::room::character::BattleCharacter;
use crate::room::map_data::{Cell, TileMap};
use crate::TEMPLATES;
use log::error;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use tools::protos::base::{ActionUnitPt, TargetPt};
use tools::templates::buff_temp::BuffTemp;

#[derive(Clone, Debug)]
pub struct Buff {
    pub id: u32,
    pub buff_temp: &'static BuffTemp,
    pub trigger_timesed: i8,   //已经触发过的次数
    pub keep_times: i8,        //剩余持续轮数
    pub scope: Vec<Direction>, //buff的作用范围
    pub permanent: bool,       //是否永久
    pub user_id: u32,          //来源的玩家id
}

impl Buff {
    pub fn get_target(&self) -> TargetType {
        let target_type = TargetType::from(self.buff_temp.target);
        target_type
    }

    pub(crate) fn sub_trigger_timesed(&mut self) {
        self.trigger_timesed -= 1;
        if self.trigger_timesed < 0 {
            self.trigger_timesed = 0;
        }
    }

    pub(crate) fn sub_keep_times(&mut self) {
        self.keep_times -= 1;
        if self.keep_times < 0 {
            self.keep_times = 0;
        }
    }
}

impl From<&'static BuffTemp> for Buff {
    fn from(bt: &'static BuffTemp) -> Self {
        let mut b = Buff {
            id: bt.id,
            trigger_timesed: bt.trigger_times as i8,
            keep_times: bt.keep_time as i8,
            buff_temp: bt,
            scope: Vec::new(),
            permanent: bt.keep_time == 0 && bt.trigger_times == 0,
            user_id: 0,
        };
        let mut v = Vec::new();
        let scope_temp = TEMPLATES.get_skill_scope_ref().get_temp(&bt.scope);
        if let Ok(scope_temp) = scope_temp {
            if !scope_temp.scope.is_empty() {
                for direction in scope_temp.scope.iter() {
                    let dir = Direction {
                        direction: &direction.direction,
                    };
                    v.push(dir);
                }
                b.scope = v;
            }
        }
        b
    }
}

impl BattleData {
    unsafe fn match_buff(
        &mut self,
        user_id: u32,
        battle_cters: *mut HashMap<u32, BattleCharacter>,
        buffs: &Vec<Buff>,
        index: u32,
        last_index: Option<usize>,
        au: &mut ActionUnitPt,
        is_pair: bool,
    ) {
        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let cell = self.tile_map.map.get(index as usize);
        let cell = cell.unwrap();
        let cell_element = cell.element;
        let cell_temp = TEMPLATES.get_cell_ref().get_temp(&cell.id).unwrap();
        for buff in buffs.iter() {
            let mut target_pt = TargetPt::new();
            target_pt.target_value.push(index);
            if is_pair {
                let last_index = last_index.unwrap();
                let last_cell = self.tile_map.map.get_mut(last_index).unwrap();
                //获得道具
                if AWARD_ITEM.contains(&buff.id) {
                    let item_id = buff.buff_temp.par1;
                    let item = TEMPLATES.get_item_ref().get_temp(&item_id);
                    if let Err(e) = item {
                        error!("{:?}", e);
                        continue;
                    }
                    let item_temp = item.unwrap();
                    let skill_id = item_temp.trigger_skill;
                    let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id);
                    if let Err(e) = skill_temp {
                        error!("{:?}", e);
                        continue;
                    }
                    let skill_temp = skill_temp.unwrap();
                    let item = Item {
                        id: item_id,
                        skill_temp,
                    };
                    target_pt.effect_type = EffectType::RewardItem as u32;
                    target_pt.effect_value = item_id;
                    //判断目标类型，若是地图块上的玩家，则判断之前那个地图块上有没有玩家，有就给他道具
                    if buff.buff_temp.target == TargetType::CellPlayer as u32 {
                        let last_cell_user =
                            battle_cters.as_mut().unwrap().get_mut(&last_cell.user_id);
                        if let Some(last_cell_user) = last_cell_user {
                            target_pt
                                .target_value
                                .push(last_cell_user.cell_index as u32);
                            au.targets.push(target_pt.clone());
                            last_cell_user.items.insert(item_id, item.clone());
                        }
                    }
                    target_pt.target_value.push(index as u32);
                    au.targets.push(target_pt.clone());
                    battle_cter.items.insert(item_id, item);
                } else if PAIR_CURE.contains(&buff.id) {
                    //配对恢复生命
                    target_pt.effect_type = EffectType::Cure as u32;
                    target_pt.effect_value = buff.buff_temp.par1;
                    if buff.buff_temp.target == TargetType::CellPlayer as u32 {
                        let last_cell_user =
                            battle_cters.as_mut().unwrap().get_mut(&last_cell.user_id);
                        if let Some(last_cell_user) = last_cell_user {
                            target_pt
                                .target_value
                                .push(last_cell_user.cell_index as u32);
                            au.targets.push(target_pt.clone());
                            last_cell_user.add_hp(buff.buff_temp.par1 as i32);
                        }
                    }
                    target_pt.target_value.push(index as u32);
                    au.targets.push(target_pt.clone());
                    //恢复生命值
                    battle_cter.add_hp(buff.buff_temp.par1 as i32);
                //todo 通知客户端
                } else if AWARD_BUFF.contains(&buff.id) {
                    //获得一个buff
                    target_pt.add_buffs.push(buff.id);
                    let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff.buff_temp.par1);
                    if let Err(e) = buff_temp {
                        error!("{:?}", e);
                        continue;
                    }
                    let buff_temp = buff_temp.unwrap();
                    let buff = Buff::from(buff_temp);
                    let target_type = TargetType::from(buff.buff_temp.target);

                    //如果目标类型是地图块上的玩家
                    if target_type == TargetType::CellPlayer {
                        let last_cell_user =
                            battle_cters.as_mut().unwrap().get_mut(&last_cell.user_id);
                        if let Some(last_cell_user) = last_cell_user {
                            last_cell_user.buffs.insert(buff.id, buff.clone());
                            target_pt
                                .target_value
                                .push(last_cell_user.cell_index as u32);
                            au.targets.push(target_pt.clone());
                        }
                    }
                    //给自己加
                    target_pt.target_value.push(index as u32);
                    au.targets.push(target_pt.clone());
                    battle_cter.buffs.insert(buff.id, buff);
                //todo 通知客户端
                } else if NEAR_ADD_CD.contains(&buff.id) {
                    //相临的玩家技能cd增加
                    target_pt.effect_type = EffectType::AddSkillCd as u32;
                    target_pt.effect_value = buff.buff_temp.par1;
                    let isize_index = index as isize;
                    for cter in self.battle_cter.values_mut() {
                        if cter.user_id == user_id {
                            continue;
                        }
                        let cter_index = cter.cell_index as isize;
                        for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                            let res = isize_index + *scope_index;
                            if res != cter_index {
                                continue;
                            }
                            cter.skills
                                .values_mut()
                                .for_each(|skill| skill.add_cd(Some(buff.buff_temp.par1 as i8)));
                        }
                        target_pt.target_value.push(cter.cell_index as u32);
                        au.targets.push(target_pt.clone());
                    }
                //todo 通知客户端
                } else if NEAR_SKILL_DAMAGE.contains(&buff.id) {
                    //相临都玩家造成技能伤害
                    target_pt.effect_type = EffectType::SkillDamage as u32;
                    target_pt.effect_value = buff.buff_temp.par1;
                    let scope_temp = TEMPLATES
                        .get_skill_scope_ref()
                        .get_temp(&buff.buff_temp.scope);
                    if let Err(e) = scope_temp {
                        error!("{:?}", e);
                        continue;
                    }
                    let scope_temp = scope_temp.unwrap();
                    let isize_index = index as isize;
                    let target_type = TargetType::from(buff.buff_temp.target);
                    let v = self
                        .cal_scope(user_id, isize_index, target_type, None, Some(scope_temp))
                        .unwrap();
                    let mut need_rank = true;
                    for user in v.iter() {
                        let cter = battle_cters.as_mut().unwrap().get_mut(user).unwrap();
                        target_pt.target_value.push(cter.cell_index as u32);
                        au.targets.push(target_pt.clone());
                        //造成技能伤害
                        self.deduct_hp(*user, buff.buff_temp.par1 as i32, need_rank);
                        need_rank = false;
                    }
                //todo 通知客户端
                } else if [9].contains(&buff.id) {
                    //处理世界块的逻辑
                    //配对属性一样的地图块+hp
                    //查看配对的cell的属性是否与角色属性匹配
                    if cell_temp.element != battle_cter.element {
                        continue;
                    }
                    target_pt.effect_type = EffectType::Cure as u32;
                    target_pt.effect_value = buff.buff_temp.par1;
                    au.targets.push(target_pt.clone());
                    //获得buff
                    battle_cter.add_hp(buff.buff_temp.par1 as i32);
                }
            }

            ///此处触发加攻击不用通知客户端
            if SAME_CELL_ELEMENT_ADD_ATTACK.contains(&buff.id) {
                if buff.buff_temp.par1 as u8 == battle_cter.element
                    && battle_cter.element == cell_element
                {
                    battle_cter.trigger_add_damage_buff(buff.id);
                }
            }
            ///翻开地图块加能量，配对加能量
            if OPEN_CELL_AND_PAIR.contains(&buff.id) {
                let mut energy = buff.buff_temp.par1;
                if is_pair {
                    energy += buff.buff_temp.par2;
                }
                battle_cter.energy += energy;
                if battle_cter.energy >= battle_cter.max_energy {
                    energy = battle_cter.energy - battle_cter.max_energy;
                    battle_cter.energy = battle_cter.max_energy;
                }
                target_pt.target_value.push(index as u32);
                target_pt.effect_type = EffectType::AddEnergy as u32;
                target_pt.effect_value = energy;
            }
        }
    }

    pub unsafe fn open_cell_trigger_buff(
        &mut self,
        user_id: u32,
        au: &mut ActionUnitPt,
        is_pair: bool,
    ) -> anyhow::Result<Option<ActionUnitPt>> {
        let battle_cters = self.battle_cter.borrow_mut() as *mut HashMap<u32, BattleCharacter>;
        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let index = battle_cter.cell_index;
        let tile_map = self.tile_map.borrow_mut() as *mut TileMap;
        let cell = tile_map.as_mut().unwrap().map.get(index).unwrap();
        let last_index = battle_cter.recently_open_cell_index;
        //匹配地图块的
        self.match_buff(
            user_id,
            battle_cters,
            &cell.buffs,
            index as u32,
            last_index,
            au,
            is_pair,
        );
        //匹配玩家身上的
        let mut buff_v = Vec::new();
        for v in battle_cter.buffs.values() {
            buff_v.push(v.clone());
        }
        self.match_buff(
            user_id,
            battle_cters,
            &buff_v,
            index as u32,
            last_index,
            au,
            is_pair,
        );
        Ok(None)
    }

    ///处理地图块额外其他buff
    pub unsafe fn trigger_cell_extra_buff(&mut self, user_id: u32, index: usize) {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;

        let _battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();

        let cell = self.tile_map.map.get_mut(index).unwrap();

        for _buff in cell.buffs.iter() {}
    }

    ///处理角色移动之后的事件
    pub unsafe fn handler_cter_move(&mut self, user_id: u32, index: usize) -> Vec<ActionUnitPt> {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let tile_map = self.tile_map.borrow_mut() as *mut TileMap;
        let cell = tile_map.as_mut().unwrap().map.get_mut(index).unwrap();
        let last_index = battle_cter.cell_index;
        //判断改地图块上面有没有角色，有的话将目标位置的玩家挪到操作玩家的位置上
        if cell.user_id > 0 {
            let target_cter = self.battle_cter.get_mut(&cell.user_id).unwrap();
            target_cter.cell_index = battle_cter.cell_index;

            let source_cell = self.tile_map.map.get_mut(last_index).unwrap();
            source_cell.user_id = target_cter.user_id;
        } else {
            //重制之前地图块上的玩家id
            let last_cell = self.tile_map.map.get_mut(last_index).unwrap();
            last_cell.user_id = 0;
        }
        //改变角色位置
        battle_cter.cell_index = index;
        cell.user_id = battle_cter.user_id;

        let mut v = Vec::new();
        let index = index as isize;

        //踩到别人到范围
        for other_cter in battle_cters.as_mut().unwrap().values_mut() {
            let cter_index = other_cter.cell_index as isize;

            for buff in other_cter.buffs.values_mut() {
                if buff.id != 1 {
                    continue;
                }
                for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                    let res = cter_index + scope_index;
                    if index != res {
                        continue;
                    }
                    self.deduct_hp(battle_cter.user_id, buff.buff_temp.par1 as i32, true);
                    let mut other_aupt = ActionUnitPt::new();
                    other_aupt.from_user = other_cter.user_id;
                    other_aupt.action_type = ActionType::Buff as u32;
                    other_aupt.action_value.push(buff.id);
                    let mut target_pt = TargetPt::new();
                    target_pt.target_value.push(battle_cter.cell_index as u32);
                    target_pt.effect_type = EffectType::SkillDamage as u32;
                    target_pt.effect_value = buff.buff_temp.par1;
                    other_aupt.targets.push(target_pt);
                    v.push(other_aupt);
                    break;
                }
                if battle_cter.is_died() {
                    break;
                }
            }
            //别人进入自己的范围触发
            if battle_cter.user_id == other_cter.user_id {
                continue;
            }
            for buff in battle_cter.buffs.values_mut() {
                if buff.id != 1 {
                    continue;
                }
                let mut need_rank = true;
                for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                    let res = index + scope_index;
                    if cter_index != res {
                        continue;
                    }
                    self.deduct_hp(other_cter.user_id, buff.buff_temp.par1 as i32, need_rank);
                    let mut self_aupt = ActionUnitPt::new();
                    self_aupt.from_user = user_id;
                    self_aupt.action_type = ActionType::Buff as u32;
                    self_aupt.action_value.push(buff.id);
                    let mut target_pt = TargetPt::new();
                    target_pt.target_value.push(other_cter.cell_index as u32);
                    target_pt.effect_type = EffectType::SkillDamage as u32;
                    target_pt.effect_value = buff.buff_temp.par1;
                    self_aupt.targets.push(target_pt);
                    v.push(self_aupt);
                    break;
                }
                need_rank = false;
            }
        }
        v
    }
}

impl Find<Buff> for Vec<Buff> {
    fn find(&self, key: usize) -> Option<&Buff> {
        for buff in self.iter() {
            if buff.id != key as u32 {
                continue;
            }
            return Some(buff);
        }
        return None;
    }

    fn find_mut(&mut self, key: usize) -> Option<&mut Buff> {
        for buff in self.iter_mut() {
            if buff.id != key as u32 {
                continue;
            }
            return Some(buff);
        }
        return None;
    }
}

impl Delete<Buff> for Vec<Buff> {
    fn delete(&mut self, key: usize) {
        for index in 0..self.len() {
            let res = self.get(index);
            if res.is_none() {
                continue;
            }
            let res = res.unwrap();
            if res.id != key as u32 {
                continue;
            }
            self.remove(index);
        }
    }
}
