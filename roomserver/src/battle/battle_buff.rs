use crate::battle::battle::{BattleData, Direction, Item};
use crate::battle::battle_enum::buff_type::{
    ATTACKED_ADD_ENERGY, AWARD_BUFF, AWARD_ITEM, CAN_NOT_MOVED, CHANGE_SKILL,
    DEFENSE_NEAR_MOVE_SKILL_DAMAGE, NEAR_ADD_CD, NEAR_SKILL_DAMAGE_PAIR, OPEN_CELL_AND_PAIR,
    PAIR_CURE, PAIR_SAME_ELEMENT_CURE, SAME_CELL_ELEMENT_ADD_ATTACK,
};
use crate::battle::battle_enum::{ActionType, EffectType, TriggerEffectType};
use crate::battle::battle_enum::{TargetType, TRIGGER_SCOPE_NEAR};
use crate::handlers::battle_handler::{Delete, Find};
use crate::room::character::BattleCharacter;
use crate::room::map_data::TileMap;
use crate::TEMPLATES;
use log::error;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::convert::TryFrom;
use tools::protos::base::{ActionUnitPt, EffectPt, TargetPt, TriggerEffectPt};
use tools::templates::buff_temp::BuffTemp;
use tools::templates::cell_temp::CellTemp;

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
        let target_type = TargetType::try_from(self.buff_temp.target as u8).unwrap();
        let target_type = TargetType::from(target_type);
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
    ///获得道具
    fn reward_item(
        &mut self,
        user_id: u32,
        buff_id: u32,
        last_cell_user_id: u32,
        battle_cters: &mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
    ) {
        let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        let item_id = buff_temp.par1;
        let item_temp = TEMPLATES.get_item_ref().get_temp(&item_id);

        if let Err(e) = item_temp {
            error!("{:?}", e);
            return;
        }
        let item_temp = item_temp.unwrap();

        let skill_id = item_temp.trigger_skill;
        let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id);
        if let Err(e) = skill_temp {
            error!("{:?}", e);
            return;
        }
        let skill_temp = skill_temp.unwrap();

        let item = Item {
            id: item_id,
            skill_temp,
        };
        let mut target_pt = TargetPt::new();
        let mut ep = EffectPt::new();
        ep.effect_type = EffectType::RewardItem as u32;
        ep.effect_value = item_id;
        target_pt.effects.push(ep);
        //判断目标类型，若是地图块上的玩家，则判断之前那个地图块上有没有玩家，有就给他道具
        if buff_temp.target == TargetType::CellPlayer as u32 {
            let last_cell_user = battle_cters.get_mut(&last_cell_user_id);
            if let Some(last_cell_user) = last_cell_user {
                target_pt
                    .target_value
                    .push(last_cell_user.cell_index as u32);
                au.targets.push(target_pt.clone());
                last_cell_user.items.insert(item_id, item.clone());
            }
        }
        let battle_cter = battle_cters.get_mut(&user_id);
        if let None = battle_cter {
            error!("battle_cter is not find!user_id:{}", user_id);
            return;
        }
        let battle_cter = battle_cter.unwrap();
        target_pt.target_value.clear();
        target_pt.target_value.push(battle_cter.cell_index as u32);
        au.targets.push(target_pt);

        battle_cter.items.insert(item_id, item);
    }

    ///匹配获得治疗
    fn pair_cure(
        &mut self,
        user_id: u32,
        buff_id: u32,
        last_cell_user_id: u32,
        _: &mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
    ) {
        let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        if buff_temp.target == TargetType::CellPlayer as u32 {
            let target_pt = self.add_hp(last_cell_user_id, buff_temp.par1 as i32);
            if let Some(target_pt) = target_pt {
                au.targets.push(target_pt);
            }
        }
        //恢复生命值
        let target_pt = self.add_hp(user_id, buff_temp.par1 as i32);
        if let Some(target_pt) = target_pt {
            au.targets.push(target_pt);
        }
    }

    ///获得buff
    fn award_buff(
        &mut self,
        user_id: u32,
        buff_id: u32,
        last_cell_user_id: u32,
        battle_cters: &mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
    ) {
        let mut target_pt = TargetPt::new();
        let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        let new_buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_temp.par1);
        if let Err(e) = new_buff_temp {
            error!("{:?}", e);
            return;
        }
        let new_buff_temp = new_buff_temp.unwrap();
        let buff = Buff::from(new_buff_temp);
        target_pt.add_buffs.push(new_buff_temp.id);
        let target_type = TargetType::try_from(buff.buff_temp.target as u8).unwrap();

        //如果目标类型是地图块上的玩家
        if target_type == TargetType::CellPlayer {
            let last_cell_user = battle_cters.get_mut(&last_cell_user_id);
            if let Some(last_cell_user) = last_cell_user {
                last_cell_user.buffs.insert(buff.id, buff.clone());
                target_pt
                    .target_value
                    .push(last_cell_user.cell_index as u32);
                au.targets.push(target_pt.clone());
            }
        }
        let battle_cter = battle_cters.get_mut(&user_id).unwrap();
        //给自己加
        target_pt.target_value.clear();
        target_pt.target_value.push(battle_cter.cell_index as u32);
        au.targets.push(target_pt);

        battle_cter.buffs.insert(buff.id, buff);
    }

    ///给附近的人添加技能cd
    fn near_add_cd(
        &mut self,
        user_id: u32,
        index: u32,
        buff_id: u32,
        battle_cters: &mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
    ) {
        let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        let mut target_pt = TargetPt::new();
        let mut ep = EffectPt::new();
        ep.effect_type = EffectType::AddSkillCd as u32;
        ep.effect_value = buff_temp.par1;
        target_pt.effects.push(ep);
        let isize_index = index as isize;
        for cter in battle_cters.values_mut() {
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
                    .for_each(|skill| skill.add_cd(Some(buff_temp.par1 as i8)));
            }
            target_pt.target_value.clear();
            target_pt.target_value.push(cter.cell_index as u32);
            au.targets.push(target_pt.clone());
        }
    }

    ///附近造成技能伤害
    fn near_skill_damage(
        &mut self,
        user_id: u32,
        index: u32,
        buff_id: u32,
        _: &mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
    ) {
        let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();

        let scope_temp = TEMPLATES.get_skill_scope_ref().get_temp(&buff_temp.scope);
        if let Err(e) = scope_temp {
            error!("{:?}", e);
            return;
        }
        let scope_temp = scope_temp.unwrap();
        let isize_index = index as isize;
        let target_type = TargetType::try_from(buff_temp.target as u8).unwrap();
        let v = self.cal_scope(user_id, isize_index, target_type, None, Some(scope_temp));
        let mut need_rank = true;
        unsafe {
            for target_user in v.iter() {
                //造成技能伤害
                let target_pt = self.deduct_hp(
                    user_id,
                    *target_user,
                    Some(buff_temp.par1 as i32),
                    need_rank,
                );

                match target_pt {
                    Ok(target_pt) => {
                        au.targets.push(target_pt);
                        need_rank = false;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
            }
        }
    }

    ///匹配同元素治疗
    fn pair_same_element_cure(
        &mut self,
        user_id: u32,
        cell_temp: &CellTemp,
        buff_id: u32,
        battle_cters: &mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
    ) {
        let battle_cter = battle_cters.get_mut(&user_id).unwrap();
        if cell_temp.element != battle_cter.element {
            return;
        }
        let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        //获得buff
        let target_pt = self.add_hp(user_id, buff_temp.par1 as i32);
        if let Some(target_pt) = target_pt {
            au.targets.push(target_pt);
        }
    }

    ///打开块和匹配
    fn open_cell_and_pair(
        &mut self,
        user_id: u32,
        buff_id: u32,
        battle_cters: &mut HashMap<u32, BattleCharacter>,
        is_pair: bool,
        au: &mut ActionUnitPt,
    ) {
        let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        let battle_cter = battle_cters.get_mut(&user_id).unwrap();
        let mut target_pt = TargetPt::new();
        target_pt.target_value.push(battle_cter.cell_index as u32);
        let mut energy = buff_temp.par1;
        if is_pair {
            energy += buff_temp.par2;
        }
        battle_cter.energy += energy;
        if battle_cter.energy > battle_cter.max_energy {
            energy = 0;
            battle_cter.energy = battle_cter.max_energy;
        }
        let mut ep = EffectPt::new();
        ep.effect_type = EffectType::AddEnergy as u32;
        ep.effect_value = energy;
        target_pt.effects.push(ep);
        au.targets.push(target_pt);
    }

    ///匹配buff
    unsafe fn match_buff(
        &mut self,
        user_id: u32,
        battle_cters: *mut HashMap<u32, BattleCharacter>,
        buffs: &Vec<Buff>,
        au: &mut ActionUnitPt,
        is_pair: bool,
    ) {
        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let index = battle_cter.cell_index as u32;
        let last_index = battle_cter.recently_open_cell_index;
        let cell = self.tile_map.map.get(index as usize);
        let cell = cell.unwrap();
        let cell_element = cell.element;
        let cell_temp = TEMPLATES.get_cell_ref().get_temp(&cell.id).unwrap();

        let cters = battle_cters.as_mut().unwrap();
        for buff in buffs.iter() {
            if is_pair {
                let last_index = last_index.unwrap();
                let last_cell = self.tile_map.map.get_mut(last_index).unwrap();
                let last_cell_user_id = last_cell.user_id;
                //获得道具
                if AWARD_ITEM.contains(&buff.id) {
                    self.reward_item(user_id, buff.id, last_cell_user_id, cters, au);
                } else if PAIR_CURE.contains(&buff.id) {
                    self.pair_cure(user_id, buff.id, last_cell_user_id, cters, au);
                } else if AWARD_BUFF.contains(&buff.id) {
                    //获得一个buff
                    self.award_buff(
                        user_id,
                        buff.id,
                        last_cell_user_id,
                        battle_cters.as_mut().unwrap(),
                        au,
                    );
                } else if NEAR_ADD_CD.contains(&buff.id) {
                    //相临的玩家技能cd增加
                    self.near_add_cd(user_id, index, buff.id, cters, au);

                //todo 通知客户端
                } else if NEAR_SKILL_DAMAGE_PAIR.contains(&buff.id) {
                    //相临都玩家造成技能伤害
                    self.near_skill_damage(user_id, index, buff.id, cters, au);

                //todo 通知客户端
                } else if PAIR_SAME_ELEMENT_CURE.contains(&buff.id) {
                    //处理世界块的逻辑
                    //配对属性一样的地图块+hp
                    //查看配对的cell的属性是否与角色属性匹配
                    self.pair_same_element_cure(user_id, cell_temp, buff.id, cters, au);
                }
            }

            //此处触发加攻击不用通知客户端
            if SAME_CELL_ELEMENT_ADD_ATTACK.contains(&buff.id) {
                if buff.buff_temp.par1 as u8 == battle_cter.element
                    && battle_cter.element == cell_element
                {
                    battle_cter.trigger_add_damage_buff(buff.id);
                }
            }
            //翻开地图块加能量，配对加能量
            if OPEN_CELL_AND_PAIR.contains(&buff.id) {
                self.open_cell_and_pair(user_id, buff.id, cters, is_pair, au);
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
        //匹配地图块的
        self.match_buff(user_id, battle_cters, &cell.buffs, au, is_pair);
        //匹配玩家身上的
        let mut buff_v = Vec::new();
        for v in battle_cter.buffs.values() {
            buff_v.push(v.clone());
        }
        self.match_buff(user_id, battle_cters, &buff_v, au, is_pair);
        Ok(None)
    }

    ///受到普通攻击触发的buff
    pub fn attacked_trigger_buffs(&mut self, user_id: u32, target_pt: &mut TargetPt) {
        let cter = self.battle_cter.get_mut(&user_id).unwrap();
        for buff in cter.buffs.clone().values() {
            let buff_id = buff.id;
            //被攻击打断技能
            if CHANGE_SKILL.contains(&buff_id) {
                cter.buffs.remove(&buff_id);
                target_pt.lost_buffs.push(buff_id);
            }

            //被攻击增加能量
            if ATTACKED_ADD_ENERGY.contains(&buff_id) && cter.max_energy > 0 {
                let mut tep = TriggerEffectPt::new();
                tep.set_field_type(TriggerEffectType::Buff as u32);
                tep.set_value(buff_id);
                let mut res = buff.buff_temp.par1;
                cter.energy += res;
                if cter.energy > cter.max_energy {
                    cter.energy = cter.max_energy;
                    res = cter.max_energy - cter.energy;
                }
                let mut ep = EffectPt::new();
                ep.effect_type = EffectType::AddEnergy.into_u32();
                ep.effect_value = buff.buff_temp.par1;
                target_pt.effects.push(ep);
            }
        }
    }

    ///处理角色移动之后的事件
    pub unsafe fn handler_cter_move(
        &mut self,
        user_id: u32,
        index: usize,
    ) -> anyhow::Result<Vec<ActionUnitPt>> {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let tile_map = self.tile_map.borrow_mut() as *mut TileMap;
        let cell = tile_map.as_mut().unwrap().map.get_mut(index).unwrap();
        let last_index = battle_cter.cell_index;
        let mut v = Vec::new();
        //判断改地图块上面有没有角色，有的话将目标位置的玩家挪到操作玩家的位置上
        if cell.user_id > 0 {
            //先判断目标位置的角色是否有不动泰山被动技能
            let target_cter = self.battle_cter.get_mut(&cell.user_id).unwrap();

            if target_cter.buffs.contains_key(&CAN_NOT_MOVED) {
                anyhow::bail!(
                    "this cter can not be move!cter_id:{},buff_id:{}",
                    target_cter.cter_id,
                    CAN_NOT_MOVED
                )
            }

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

        let index = index as isize;

        for other_cter in battle_cters.as_mut().unwrap().values_mut() {
            let cter_index = other_cter.cell_index as isize;
            //踩到别人到范围
            for buff in other_cter.buffs.values_mut() {
                if !DEFENSE_NEAR_MOVE_SKILL_DAMAGE.contains(&buff.id) {
                    continue;
                }
                for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                    let res = cter_index + scope_index;
                    if index != res {
                        continue;
                    }
                    let target_pt = self.deduct_hp(
                        other_cter.user_id,
                        battle_cter.user_id,
                        Some(buff.buff_temp.par1 as i32),
                        true,
                    );
                    match target_pt {
                        Ok(target_pt) => {
                            let mut other_aupt = ActionUnitPt::new();
                            other_aupt.from_user = other_cter.user_id;
                            other_aupt.action_type = ActionType::Buff as u32;
                            other_aupt.action_value.push(buff.id);
                            other_aupt.targets.push(target_pt);
                            v.push(other_aupt);
                            break;
                        }
                        Err(e) => error!("{:?}", e),
                    }
                }
                if battle_cter.is_died() {
                    break;
                }
            }
            //别人进入自己的范围触发
            //现在没有种buff，先注释代码
            // if battle_cter.user_id == other_cter.user_id {
            //     continue;
            // }
            // for buff in battle_cter.buffs.values_mut() {
            //     if !DEFENSE_NEAR_MOVE_SKILL_DAMAGE.contains(&buff.id) {
            //         continue;
            //     }
            //     let mut need_rank = true;
            //     for scope_index in TRIGGER_SCOPE_NEAR.iter() {
            //         let res = index + scope_index;
            //         if cter_index != res {
            //             continue;
            //         }
            //         let target_pt = self.deduct_hp(
            //             battle_cter.user_id,
            //             other_cter.user_id,
            //             Some(buff.buff_temp.par1 as i32),
            //             need_rank,
            //         );
            //         match target_pt {
            //             Ok(target_pt) => {
            //                 let mut self_aupt = ActionUnitPt::new();
            //                 self_aupt.from_user = user_id;
            //                 self_aupt.action_type = ActionType::Buff as u32;
            //                 self_aupt.action_value.push(buff.id);
            //                 self_aupt.targets.push(target_pt);
            //                 v.push(self_aupt);
            //                 break;
            //             }
            //             Err(e) => error!("{:?}", e),
            //         }
            //     }
            //     need_rank = false;
            // }
        }
        Ok(v)
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
