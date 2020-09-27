use crate::battle::battle::{BattleData, Direction};
use crate::battle::battle_enum::buff_type::{
    AWARD_BUFF, AWARD_ITEM, NEAR_ADD_CD, NEAR_SKILL_DAMAGE_PAIR, OPEN_CELL_AND_PAIR_ADD_ENERGY,
    PAIR_CLEAN_SKILL_CD, PAIR_CURE, PAIR_SAME_ELEMENT_ADD_ATTACK, PAIR_SAME_ELEMENT_CURE,
};
use crate::battle::battle_enum::EffectType;
use crate::battle::battle_enum::{TargetType, TRIGGER_SCOPE_NEAR};
use crate::handlers::battle_handler::{Delete, Find};
use crate::room::character::BattleCharacter;
use crate::room::map_data::TileMap;
use crate::TEMPLATES;
use log::{error, warn};
use std::borrow::BorrowMut;
use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::convert::TryFrom;
use tools::protos::base::{ActionUnitPt, EffectPt, TargetPt, TriggerEffectPt};
use tools::templates::buff_temp::BuffTemp;

#[derive(Clone, Debug)]
pub struct Buff {
    pub id: u32,
    pub buff_temp: &'static BuffTemp,
    pub trigger_timesed: i8,       //已经触发过的次数
    pub keep_times: i8,            //剩余持续轮数
    pub scope: Vec<Direction>,     //buff的作用范围
    pub permanent: bool,           //是否永久
    pub from_user: Option<u32>,    //来源的玩家id
    pub from_skill: Option<u32>,   //来源的技能id
    pub turn_index: Option<usize>, //生效于turn_index
}

impl Buff {
    pub fn new(
        temp: &'static BuffTemp,
        turn_index: Option<usize>,
        from_user: Option<u32>,
        from_skill: Option<u32>,
    ) -> Self {
        let mut buff = Buff::from(temp);
        buff.turn_index = turn_index;
        buff.from_user = from_user;
        buff.from_skill = from_skill;
        buff
    }

    pub(crate) fn sub_trigger_timesed(&mut self) {
        self.trigger_timesed -= 1;
        if self.trigger_timesed < 0 {
            self.trigger_timesed = 0;
        }
    }

    pub(crate) fn sub_keep_times(&mut self) {
        if self.buff_temp.keep_time > 0 {
            self.keep_times -= 1;
        }
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
            from_user: None,
            from_skill: None,
            turn_index: None,
        };
        let mut v = Vec::new();
        let scope_temp = TEMPLATES.get_skill_scope_temp_mgr_ref().get_temp(&bt.scope);
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
        from_user: Option<u32>,
        user_id: u32,
        buff_id: u32,
        last_map_cell_user_id: u32,
        battle_cters: &mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
    ) {
        let buff_temp = TEMPLATES.get_buff_temp_mgr_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        let item_id = buff_temp.par1;

        let battle_cter = battle_cters.get_mut(&user_id);
        if let None = battle_cter {
            error!("battle_cter is not find!user_id:{}", user_id);
            return;
        }
        let battle_cter = battle_cter.unwrap();
        let res = battle_cter.add_item(item_id);
        if let Err(e) = res {
            warn!("{:?}", e);
            return;
        }
        let target_pt = self.build_target_pt(
            from_user,
            user_id,
            EffectType::RewardItem,
            item_id,
            Some(buff_id),
        );
        match target_pt {
            Ok(target_pt) => {
                au.targets.push(target_pt);
            }
            Err(e) => {
                warn!("{:?}", e);
                return;
            }
        }
        //判断目标类型，若是地图块上的玩家，则判断之前那个地图块上有没有玩家，有就给他道具
        if buff_temp.target == TargetType::MapCellPlayer.into_u8() {
            let last_map_cell_user = battle_cters.get_mut(&last_map_cell_user_id);
            if let Some(last_map_cell_user) = last_map_cell_user {
                let res = last_map_cell_user.add_item(item_id);
                if let Err(e) = res {
                    warn!("{:?}", e);
                    return;
                }
                let target_pt = self.build_target_pt(
                    from_user,
                    last_map_cell_user.base_attr.user_id,
                    EffectType::RewardItem,
                    item_id,
                    Some(buff_id),
                );
                if let Err(e) = target_pt {
                    warn!("{:?}", e);
                    return;
                }
                au.targets.push(target_pt.unwrap());
            }
        }
    }

    ///匹配获得治疗
    fn pair_cure(
        &mut self,
        from_user: Option<u32>,
        user_id: u32,
        buff_id: u32,
        last_map_cell_user_id: u32,
        _: &mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
    ) {
        let buff_temp = TEMPLATES.get_buff_temp_mgr_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        if buff_temp.target == TargetType::MapCellPlayer.into_u8() {
            let target_pt = self.add_hp(
                from_user,
                last_map_cell_user_id,
                buff_temp.par1 as i16,
                Some(buff_id),
            );

            match target_pt {
                Ok(target_pt) => {
                    au.targets.push(target_pt);
                }
                Err(e) => warn!("{:?}", e),
            }
        }
        //恢复生命值
        let target_pt = self.add_hp(from_user, user_id, buff_temp.par1 as i16, Some(buff_id));
        match target_pt {
            Ok(target_pt) => {
                au.targets.push(target_pt);
            }
            Err(e) => warn!("{:?}", e),
        }
    }

    ///获得buff
    fn award_buff(
        &mut self,
        from_user: Option<u32>,
        from_skill: Option<u32>,
        target_user: u32,
        buff_id: u32,
        last_map_cell_user_id: u32,
        battle_cters: &mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
    ) {
        let mut target_pt = TargetPt::new();
        let buff_temp = TEMPLATES.get_buff_temp_mgr_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        let new_buff_temp = TEMPLATES.get_buff_temp_mgr_ref().get_temp(&buff_temp.par1);
        if let Err(e) = new_buff_temp {
            error!("{:?}", e);
            return;
        }
        let new_buff_temp = new_buff_temp.unwrap();
        let buff = Buff::new(
            new_buff_temp,
            Some(self.next_turn_index),
            from_user,
            from_skill,
        );
        target_pt.add_buffs.push(new_buff_temp.id);
        let target_type = TargetType::try_from(buff.buff_temp.target as u8).unwrap();

        //如果目标类型是地图块上的玩家
        if target_type == TargetType::MapCellPlayer {
            let last_map_cell_user = battle_cters.get_mut(&last_map_cell_user_id);
            if let Some(last_map_cell_user) = last_map_cell_user {
                last_map_cell_user
                    .battle_buffs
                    .buffs
                    .insert(buff.id, buff.clone());
                target_pt
                    .target_value
                    .push(last_map_cell_user.get_map_cell_index() as u32);
                au.targets.push(target_pt.clone());
            }
        }
        let battle_cter = battle_cters.get_mut(&target_user).unwrap();
        //给自己加
        target_pt.target_value.clear();
        target_pt
            .target_value
            .push(battle_cter.get_map_cell_index() as u32);
        au.targets.push(target_pt);

        battle_cter.battle_buffs.buffs.insert(buff.id, buff);
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
        let buff_temp = TEMPLATES.get_buff_temp_mgr_ref().get_temp(&buff_id);
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
            if cter.get_user_id() == user_id {
                continue;
            }
            if cter.is_died() {
                continue;
            }
            let cter_index = cter.get_map_cell_index() as isize;
            for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                let res = isize_index + *scope_index;
                if res != cter_index {
                    continue;
                }
                if cter.base_attr.max_energy > 0 {
                    continue;
                }
                cter.skills
                    .values_mut()
                    .for_each(|skill| skill.add_cd(Some(buff_temp.par1 as i8)));
            }
            target_pt.target_value.clear();
            target_pt
                .target_value
                .push(cter.get_map_cell_index() as u32);
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
        let buff_temp = TEMPLATES.get_buff_temp_mgr_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();

        let scope_temp = TEMPLATES
            .get_skill_scope_temp_mgr_ref()
            .get_temp(&buff_temp.scope);
        if let Err(e) = scope_temp {
            error!("{:?}", e);
            return;
        }
        let scope_temp = scope_temp.unwrap();
        let isize_index = index as isize;
        let target_type = TargetType::try_from(buff_temp.target as u8).unwrap();
        let (_, v) = self.cal_scope(user_id, isize_index, target_type, None, Some(scope_temp));
        let mut need_rank = true;
        unsafe {
            for target_user in v.iter() {
                //造成技能伤害
                let target_pt = self.deduct_hp(
                    user_id,
                    *target_user,
                    Some(buff_temp.par1 as i16),
                    need_rank,
                );

                match target_pt {
                    Ok(target_pt) => {
                        au.targets.push(target_pt);
                    }
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
                need_rank = false;
            }
        }
    }

    ///匹配清空指定技能cd
    fn pair_clean_skill_cd(
        &mut self,
        user_id: u32,
        buff_id: u32,
        skill_id: u32,
        au: &mut ActionUnitPt,
    ) {
        let cter = self.get_battle_cter_mut(Some(user_id), true);
        if let Err(e) = cter {
            error!("{:?}", e);
            return;
        }
        let cter = cter.unwrap();
        let skill = cter.skills.get_mut(&skill_id);
        if let None = skill {
            warn!("this cter has no this skill!skill_id:{}", skill_id);
            return;
        }
        let skill = skill.unwrap();
        skill.cd_times = 0;
        let mut target_pt = TargetPt::new();
        target_pt
            .target_value
            .push(cter.get_map_cell_index() as u32);
        let mut tep = TriggerEffectPt::new();
        tep.buff_id = buff_id;
        target_pt.passiveEffect.push(tep);
        au.targets.push(target_pt);
    }

    ///匹配同元素治疗
    fn pair_same_element_cure(
        &mut self,
        from_user: Option<u32>,
        target_user: u32,
        map_cell_element: u8,
        buff_id: u32,
        battle_cters: &mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
    ) {
        let battle_cter = battle_cters.get_mut(&target_user).unwrap();
        if map_cell_element != battle_cter.base_attr.element {
            return;
        }
        let buff_temp = TEMPLATES.get_buff_temp_mgr_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        //获得buff
        let target_pt = self.add_hp(from_user, target_user, buff_temp.par1 as i16, Some(buff_id));
        match target_pt {
            Ok(target_pt) => {
                au.targets.push(target_pt);
            }
            Err(e) => {
                warn!("{:?}", e);
            }
        }
    }

    ///打开块和匹配
    fn open_map_cell_and_pair(
        &mut self,
        from_user: Option<u32>,
        target_user: u32,
        target_battle: &mut BattleCharacter,
        buff_id: u32,
        is_pair: bool,
        au: &mut ActionUnitPt,
    ) {
        let buff_temp = TEMPLATES.get_buff_temp_mgr_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        let mut energy = buff_temp.par1 as u8;
        if is_pair {
            energy += buff_temp.par2 as u8;
        }
        target_battle.add_energy(energy as i8);
        let mut target_pt = TargetPt::new();
        target_pt
            .target_value
            .push(target_battle.get_map_cell_index() as u32);

        if from_user.is_some() && from_user.unwrap() == target_user {
            let mut tep = TriggerEffectPt::new();
            tep.buff_id = buff_id;
            tep.set_field_type(EffectType::AddEnergy.into_u32());
            tep.set_value(energy as u32);
            target_pt.passiveEffect.push(tep);
        } else {
            let mut ep = EffectPt::new();
            ep.set_effect_type(EffectType::AddEnergy.into_u32());
            ep.set_effect_value(energy as u32);
            target_pt.effects.push(ep);
        }
        au.targets.push(target_pt);
    }

    ///匹配打开地图块触发buff
    ///from_user: buff的来源玩家id
    ///buffs: Values<u32, Buff>, buff列表
    ///match_user: u32,匹配的玩家id
    ///open_user: u32,打开地图块的玩家id
    ///battle_cters: *mut HashMap<u32, BattleCharacter>,裸指针
    ///au: &mut ActionUnitPt,proto
    ///is_pair: bool,是否配对了
    unsafe fn match_open_map_cell_buff(
        &mut self,
        from_user: Option<u32>,
        buffs: Values<u32, Buff>,
        match_user: u32,
        open_user: u32,
        battle_cters: *mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
        is_pair: bool,
    ) {
        let cter = battle_cters.as_mut().unwrap().get_mut(&match_user);
        if let None = cter {
            error!("battle_cter not find!user_id:{}", match_user);
            return;
        }
        let cter = cter.unwrap();

        let open_cter = battle_cters.as_mut().unwrap().get_mut(&open_user);
        if let None = open_cter {
            error!("battle_cter not find!user_id:{}", open_user);
            return;
        }
        let open_cter = open_cter.unwrap();

        let last_index = open_cter.index_data.last_map_cell_index;
        let cters = battle_cters.as_mut().unwrap();
        let index = open_cter.get_map_cell_index() as u32;
        let map_cell = self.tile_map.map_cells.get(index as usize).unwrap();
        let map_cell_element = map_cell.element;
        for buff in buffs {
            //如果匹配的人和开地图块的人是同一个人
            if open_user == match_user {
                let last_map_cell_user_id;
                if let Some(last_index) = last_index {
                    let last_map_cell = self.tile_map.map_cells.get_mut(last_index);
                    if let Some(last_map_cell) = last_map_cell {
                        last_map_cell_user_id = last_map_cell.user_id;
                    } else {
                        last_map_cell_user_id = 0;
                    }
                } else {
                    last_map_cell_user_id = 0;
                }

                if is_pair {
                    //获得道具
                    if AWARD_ITEM.contains(&buff.id) {
                        self.reward_item(
                            from_user,
                            match_user,
                            buff.id,
                            last_map_cell_user_id,
                            cters,
                            au,
                        );
                    } else if PAIR_CURE.contains(&buff.id) {
                        self.pair_cure(
                            from_user,
                            match_user,
                            buff.id,
                            last_map_cell_user_id,
                            cters,
                            au,
                        );
                    } else if AWARD_BUFF.contains(&buff.id) {
                        //获得一个buff
                        self.award_buff(
                            from_user,
                            None,
                            match_user,
                            buff.id,
                            last_map_cell_user_id,
                            battle_cters.as_mut().unwrap(),
                            au,
                        );
                    } else if NEAR_ADD_CD.contains(&buff.id) {
                        //相临的玩家技能cd增加
                        self.near_add_cd(match_user, index, buff.id, cters, au);
                    } else if NEAR_SKILL_DAMAGE_PAIR.contains(&buff.id) {
                        //相临都玩家造成技能伤害
                        self.near_skill_damage(match_user, index, buff.id, cters, au);
                    } else if PAIR_SAME_ELEMENT_CURE.contains(&buff.id) {
                        //处理世界块的逻辑
                        //配对属性一样的地图块+hp
                        //查看配对的map_cell的属性是否与角色属性匹配
                        self.pair_same_element_cure(
                            from_user,
                            match_user,
                            map_cell_element,
                            buff.id,
                            cters,
                            au,
                        );
                    } else if PAIR_CLEAN_SKILL_CD.contains(&buff.id) {
                        //匹配了刷新指定技能cd
                        let skill_id = buff.buff_temp.par1;
                        self.pair_clean_skill_cd(open_user, buff.id, skill_id, au);
                    }
                }
                //翻开地图块加能量，配对加能量
                if OPEN_CELL_AND_PAIR_ADD_ENERGY.contains(&buff.id) {
                    self.open_map_cell_and_pair(from_user, match_user, cter, buff.id, is_pair, au);
                }
            }

            if is_pair {
                //匹配属性一样的地图块+攻击
                if PAIR_SAME_ELEMENT_ADD_ATTACK.contains(&buff.id) {
                    //此处触发加攻击不用通知客户端
                    let buff_element = buff.buff_temp.par1 as u8;
                    let cter_element = cter.base_attr.element;
                    if buff_element == cter_element && cter_element == map_cell_element {
                        cter.trigger_add_damage_buff(buff.id);
                    }
                }
            }
        }
    }

    ///匹配buff
    pub unsafe fn trigger_open_map_cell_buff(
        &mut self,
        map_cell_index: Option<usize>,
        user_id: u32,
        battle_cters: *mut HashMap<u32, BattleCharacter>,
        au: &mut ActionUnitPt,
        is_pair: bool,
    ) {
        let cters = battle_cters.as_mut().unwrap();
        if map_cell_index.is_none() {
            //匹配其他玩家身上的
            for cter in cters.values_mut() {
                if cter.is_died() {
                    continue;
                }
                self.match_open_map_cell_buff(
                    Some(cter.get_user_id()),
                    cter.battle_buffs.buffs.values(),
                    cter.get_user_id(),
                    user_id,
                    battle_cters,
                    au,
                    is_pair,
                );
            }
        } else {
            let tail_map_ptr = self.tile_map.borrow_mut() as *mut TileMap;
            let map_cell = tail_map_ptr
                .as_ref()
                .unwrap()
                .map_cells
                .get(map_cell_index.unwrap())
                .unwrap();
            for cter in cters.values_mut() {
                if cter.is_died() {
                    continue;
                }
                self.match_open_map_cell_buff(
                    None,
                    map_cell.buffs.values(),
                    cter.get_user_id(),
                    user_id,
                    battle_cters,
                    au,
                    is_pair,
                );
            }
        }
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
