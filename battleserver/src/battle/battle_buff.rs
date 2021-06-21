use crate::battle::battle::{BattleData, Direction};
use crate::battle::battle_enum::buff_type::{
    AWARD_BUFF, AWARD_ITEM, NEAR_ADD_CD, NEAR_SKILL_DAMAGE_PAIR, PAIR_CLEAN_SKILL_CD, PAIR_CURE,
    PAIR_SAME_ELEMENT_ADD_ATTACK, PAIR_SAME_ELEMENT_CURE,
};
use crate::battle::battle_enum::EffectType;
use crate::battle::battle_enum::{TargetType, TRIGGER_SCOPE_NEAR};
use crate::handlers::battle_handler::{Delete, Find};
use crate::room::map_data::MapCell;
use crate::TEMPLATES;
use log::{error, warn};
use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use tools::protos::base::{ActionUnitPt, EffectPt, TargetPt, TriggerEffectPt};
use tools::templates::buff_temp::BuffTemp;

use super::battle_enum::buff_type::PAIR_SAME_ELEMENT_CLEAN_OR_SUB_SKILL_CD;
use super::battle_player::BattlePlayer;

#[derive(Clone, Debug)]
pub struct Buff {
    id: u32,
    pub function_id: u32, //功能id
    pub buff_temp: &'static BuffTemp,
    pub trigger_timesed: i8,           //已经触发过的次数
    pub keep_times: i8,                //剩余持续轮数
    pub scope: Vec<Direction>,         //buff的作用范围
    pub permanent: bool,               //是否永久
    pub from_user: Option<u32>,        //来源的玩家id
    pub from_skill: Option<u32>,       //来源的技能id
    pub turn_index: Option<usize>,     //生效于turn_index
    pub trap_view_users: HashSet<u32>, //陷阱可见玩家
}

impl Buff {
    pub fn new(
        temp: &'static BuffTemp,
        turn_index: Option<usize>,
        from_user: Option<u32>,
        from_skill: Option<u32>,
    ) -> Self {
        let mut buff = Buff::from(temp);
        if temp.keep_time > 0 {
            buff.turn_index = turn_index;
        }
        buff.from_user = from_user;
        buff.from_skill = from_skill;
        buff
    }

    pub fn get_id(&self) -> u32 {
        self.id
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
            function_id: bt.function_id,
            trigger_timesed: bt.trigger_times as i8,
            keep_times: bt.keep_time as i8,
            buff_temp: bt,
            scope: Vec::new(),
            permanent: bt.keep_time == 0 && bt.trigger_times == 0,
            from_user: None,
            from_skill: None,
            turn_index: None,
            trap_view_users: HashSet::new(),
        };
        let mut v = Vec::new();
        let scope_temp = TEMPLATES.skill_scope_temp_mgr().get_temp(&bt.scope);
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
        au: &mut ActionUnitPt,
    ) {
        let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        let item_id = buff_temp.par1;

        let battle_player = self.battle_player.get_mut(&user_id);
        if let None = battle_player {
            error!("battle_player is not find!user_id:{}", user_id);
            return;
        }
        let battle_player = battle_player.unwrap();
        let res = battle_player.cter.add_item(item_id);
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
            let last_map_cell_user = self.battle_player.get_mut(&last_map_cell_user_id);
            if let None = last_map_cell_user {
                return;
            }
            let last_map_cell_user = last_map_cell_user.unwrap();
            let res = last_map_cell_user.cter.add_item(item_id);
            if let Err(e) = res {
                warn!("{:?}", e);
                return;
            }
            let target_pt = self.build_target_pt(
                from_user,
                last_map_cell_user_id,
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

    ///匹配获得治疗
    fn pair_cure(
        &mut self,
        from_user: Option<u32>,
        user_id: u32,
        buff_id: u32,
        last_map_cell_user_id: u32,
        au: &mut ActionUnitPt,
    ) {
        let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
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
        au: &mut ActionUnitPt,
    ) {
        let mut target_pt = TargetPt::new();
        let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        let new_buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_temp.par1);
        if let Err(e) = new_buff_temp {
            error!("{:?}", e);
            return;
        }
        let new_buff_temp = new_buff_temp.unwrap();
        target_pt.add_buffs.push(new_buff_temp.id);
        let target_type = TargetType::try_from(new_buff_temp.target as u8).unwrap();

        //如果目标类型是地图块上的玩家
        if target_type == TargetType::MapCellPlayer {
            let last_map_cell_user = self.battle_player.get_mut(&last_map_cell_user_id);
            if let Some(last_map_cell_user) = last_map_cell_user {
                last_map_cell_user.cter.add_buff(
                    from_user,
                    from_skill,
                    new_buff_temp.id,
                    Some(self.next_turn_index),
                );
                let last_map_cell_user_index = last_map_cell_user.cter.get_map_cell_index() as u32;
                target_pt.target_value.push(last_map_cell_user_index);
                au.targets.push(target_pt.clone());
            }
        }
        let battle_player = self.battle_player.get_mut(&target_user);
        if let None = battle_player {
            warn!("can not find battle_cter!cter_id={}", target_user);
        }
        let battle_player = battle_player.unwrap();
        let battle_cter_index = battle_player.cter.get_map_cell_index() as u32;
        //给自己加
        target_pt.target_value.clear();
        target_pt.target_value.push(battle_cter_index);
        au.targets.push(target_pt);

        battle_player.cter.add_buff(
            from_user,
            from_skill,
            new_buff_temp.id,
            Some(self.next_turn_index),
        )
    }

    ///给附近的人添加技能cd
    fn near_add_cd(&mut self, user_id: u32, index: u32, buff_id: u32, au: &mut ActionUnitPt) {
        let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
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
        for battle_player in self.battle_player.values_mut() {
            if battle_player.get_user_id() == user_id {
                continue;
            }
            if battle_player.is_died() {
                continue;
            }
            let cter_index = battle_player.cter.get_map_cell_index() as isize;
            for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                let res = isize_index + *scope_index;
                if res != cter_index {
                    continue;
                }
                if battle_player.cter.base_attr.max_energy > 0 {
                    continue;
                }
                battle_player
                    .cter
                    .skills
                    .values_mut()
                    .for_each(|skill| skill.add_cd(buff_temp.par1 as i8));
            }
            target_pt.target_value.clear();
            target_pt.target_value.push(cter_index as u32);
            au.targets.push(target_pt.clone());
        }
    }

    ///附近造成技能伤害
    fn near_skill_damage(&mut self, user_id: u32, index: u32, buff_id: u32, au: &mut ActionUnitPt) {
        let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();

        let scope_temp = TEMPLATES.skill_scope_temp_mgr().get_temp(&buff_temp.scope);
        if let Err(e) = scope_temp {
            error!("{:?}", e);
            return;
        }
        let scope_temp = scope_temp.unwrap();
        let isize_index = index as isize;
        let target_type = TargetType::try_from(buff_temp.target as u8).unwrap();
        let (_, v) = self.cal_scope(user_id, isize_index, target_type, None, Some(scope_temp));
        let mut is_last_one = false;
        unsafe {
            for index in 0..v.len() {
                if index == v.len() - 1 {
                    is_last_one = true;
                }
                let target_user = v.get(index).unwrap();
                let target_pt = self.new_target_pt(*target_user);
                if let Err(e) = target_pt {
                    error!("{:?}", e);
                    continue;
                }
                let mut target_pt = target_pt.unwrap();
                //造成技能伤害
                let res = self.deduct_hp(
                    user_id,
                    *target_user,
                    Some(buff_temp.par1 as i16),
                    &mut target_pt,
                    is_last_one,
                );
                match res {
                    Ok(_) => {
                        au.targets.push(target_pt);
                    }
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
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
        let battle_player = self.get_battle_player_mut(Some(user_id), true);
        if let Err(e) = battle_player {
            error!("{:?}", e);
            return;
        }
        let battle_player = battle_player.unwrap();
        let skill = battle_player.cter.skills.get_mut(&skill_id);
        if let None = skill {
            warn!(
                "skill is not find!skill_id:{},cter_id:{},user_id:{}",
                skill_id, battle_player.cter.base_attr.cter_id, user_id
            );
            return;
        }
        let skill = skill.unwrap();
        skill.cd_times = 0;
        let cter_index = battle_player.get_map_cell_index() as u32;
        let mut target_pt = TargetPt::new();
        target_pt.target_value.push(cter_index);
        let mut tep = TriggerEffectPt::new();
        let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            warn!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        tep.buff_id = buff_id;
        tep.set_field_type(EffectType::RefreshSkillCd.into_u32());
        tep.set_value(buff_temp.par1);
        target_pt.passiveEffect.push(tep);
        au.targets.push(target_pt);
    }

    ///匹配同元素治疗，不同就减cd
    fn pair_same_element_clean_or_sub_skill_cd(
        &mut self,
        user_id: u32,
        map_cell_element: u8,
        buff_id: u32,
        au: &mut ActionUnitPt,
    ) {
        let res = self.get_battle_player_mut(Some(user_id), true);
        if let Err(_) = res {
            return;
        }
        let battle_player = res.unwrap();
        let buff = battle_player.cter.battle_buffs.get_buff(buff_id);
        if let None = buff {
            return;
        }
        let buff = buff.unwrap();

        let par1 = buff.buff_temp.par1 as u8;
        let par2 = buff.buff_temp.par1 as i8;
        let cter_index = battle_player.get_map_cell_index() as u32;
        let mut target_pt = TargetPt::new();
        target_pt.target_value.push(cter_index);
        let mut tep = TriggerEffectPt::new();
        tep.buff_id = buff_id;
        //如果匹配的元素相同就清空技能cd
        if par1 == map_cell_element {
            tep.set_field_type(EffectType::RefreshSkillCd.into_u32());
            battle_player.cter.clean_skill_cd();
        } else {
            //减cd
            battle_player.cter.sub_skill_cd(Some(par2));
            tep.set_field_type(EffectType::SubSkillCd.into_u32());
            tep.set_value(par1 as u32);
            battle_player.cter.sub_skill_cd(Some(par1 as i8));
        }
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
        au: &mut ActionUnitPt,
    ) {
        let battle_player = self.battle_player.get(&target_user);
        if battle_player.is_none() {
            return;
        }
        let battle_player = battle_player.unwrap();
        if map_cell_element != battle_player.cter.base_attr.element {
            return;
        }
        let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
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

    ///移动和匹配
    pub fn manual_move_and_pair(
        &mut self,
        from_user: Option<u32>,
        target_user: u32,
        buff_id: u32,
        is_pair: bool,
        au: &mut ActionUnitPt,
    ) {
        let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        let mut energy = buff_temp.par1 as u8;
        if is_pair {
            energy += buff_temp.par2 as u8;
        }
        let target_player = self.battle_player.get_mut(&target_user);
        if target_player.is_none() {
            return;
        }
        let target_player = target_player.unwrap();
        target_player.cter.add_energy(energy as i8);

        let target_battle_index = target_player.cter.get_map_cell_index() as u32;

        let mut target_pt = TargetPt::new();
        target_pt.target_value.push(target_battle_index);

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

    //打开地图快触发buff for player
    fn match_open_map_cell_buff_for_user(
        &mut self,
        open_user: u32,
        au: &mut ActionUnitPt,
        is_pair: bool,
    ) {
        let battle_players_ptr = self.battle_player.borrow_mut() as *mut HashMap<u32, BattlePlayer>;
        unsafe {
            let open_player = battle_players_ptr.as_mut().unwrap().get_mut(&open_user);
            if let None = open_player {
                error!("battle_player not find!user_id:{}", open_user);
                return;
            }
            let open_player = open_player.unwrap();
            //玩家当前为止
            let index = open_player.cter.get_map_cell_index() as u32;

            //找出玩家上一个地图快为止的玩家id
            let last_index = open_player.cter.index_data.last_map_cell_index;
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
            let map_cell = self.tile_map.map_cells.get(index as usize).unwrap();
            let map_cell_element = map_cell.element;
            let mut buff_function_id;
            let mut buff_id;

            //匹配自己翻开的
            for buff in open_player.cter.battle_buffs.buffs().values() {
                buff_function_id = buff.function_id;
                buff_id = buff.id;
                if is_pair {
                    //获得道具
                    if AWARD_ITEM.contains(&buff_function_id) {
                        self.reward_item(
                            Some(open_user),
                            open_user,
                            buff_id,
                            last_map_cell_user_id,
                            au,
                        );
                    } else if PAIR_SAME_ELEMENT_CURE == buff_function_id {
                        //处理世界块的逻辑
                        //配对属性一样的地图块+hp
                        //查看配对的map_cell的属性是否与角色属性匹配
                        self.pair_same_element_cure(
                            Some(open_user),
                            open_user,
                            map_cell_element,
                            buff_id,
                            au,
                        );
                    } else if PAIR_CLEAN_SKILL_CD == buff_function_id {
                        //匹配了刷新指定技能cd
                        let skill_id = buff.buff_temp.par1;
                        self.pair_clean_skill_cd(open_user, buff_id, skill_id, au);
                    } else if PAIR_SAME_ELEMENT_CLEAN_OR_SUB_SKILL_CD == buff_function_id {
                        self.pair_same_element_clean_or_sub_skill_cd(
                            open_user,
                            map_cell_element,
                            buff_id,
                            au,
                        );
                    }
                }
            }

            //匹配其他玩家的
            let mut match_user;
            for battle_player in self.battle_player.values_mut() {
                if battle_player.is_died() {
                    continue;
                }
                match_user = battle_player.get_user_id();
                for buff in battle_player.cter.battle_buffs.buffs().values() {
                    buff_function_id = buff.function_id;
                    buff_id = buff.id;
                    //匹配属性一样的地图块+攻击
                    if PAIR_SAME_ELEMENT_ADD_ATTACK == buff_function_id {
                        let buff_element = buff.buff_temp.par1 as u8;
                        let from_user = match_user;
                        //先清除
                        let player = battle_players_ptr
                            .as_mut()
                            .unwrap()
                            .get_mut(&match_user)
                            .unwrap();
                        player.cter.remove_damage_buff(buff_id);
                        //此处触发加攻击不用通知客户端
                        let res = self.tile_map.pair_element_map_cells(buff_element);
                        let res = res.len() / 2;
                        if res == 0 {
                            return;
                        }
                        //再添加
                        for _ in 0..res {
                            player.cter.add_buff(
                                Some(from_user),
                                None,
                                buff_id,
                                Some(self.next_turn_index),
                            );
                        }
                    }
                }
            }
        }
    }

    ///匹配打开地图块触发buff for map_cell
    ///au: &mut ActionUnitPt,proto
    ///is_pair: bool,是否配对了
    fn match_open_map_cell_buff_for_map_cell(
        &mut self,
        open_user: u32,
        map_cell_index: usize,
        au: &mut ActionUnitPt,
        is_pair: bool,
    ) {
        let open_player = self.battle_player.get_mut(&open_user);
        if let None = open_player {
            error!("battle_player not find!user_id:{}", open_user);
            return;
        }
        let open_player = open_player.unwrap();

        let map_cell = self.tile_map.map_cells.get(map_cell_index);
        if let None = map_cell {
            warn!("could not find map_cell!index:{}", map_cell_index);
            return;
        }
        let map_cell = map_cell.unwrap();
        if map_cell.buffs.is_empty() {
            return;
        }
        let map_cell_ptr = map_cell as *const MapCell;

        let last_index = open_player.cter.index_data.last_map_cell_index;
        let last_map_cell_user_id;
        if let Some(last_index) = last_index {
            let last_map_cell = self.tile_map.map_cells.get(last_index);
            match last_map_cell {
                Some(last_map_cell) => {
                    last_map_cell_user_id = last_map_cell.user_id;
                }
                None => last_map_cell_user_id = 0,
            }
        } else {
            last_map_cell_user_id = 0;
        }
        let mut buff_function_id;
        let mut buff_id;
        unsafe {
            let map_cell_ref = map_cell_ptr.as_ref().unwrap();
            for buff in map_cell_ref.buffs.values() {
                buff_id = buff.id;
                buff_function_id = buff.function_id;
                if is_pair {
                    //获得道具
                    if AWARD_ITEM.contains(&buff_function_id) {
                        self.reward_item(
                            Some(open_user),
                            open_user,
                            buff_id,
                            last_map_cell_user_id,
                            au,
                        );
                    } else if PAIR_CURE == buff_function_id {
                        self.pair_cure(
                            Some(open_user),
                            open_user,
                            buff_id,
                            last_map_cell_user_id,
                            au,
                        );
                    } else if AWARD_BUFF == buff_function_id {
                        //获得一个buff
                        self.award_buff(
                            Some(open_user),
                            None,
                            open_user,
                            buff_id,
                            last_map_cell_user_id,
                            au,
                        );
                    } else if NEAR_ADD_CD == buff_function_id {
                        //相临的玩家技能cd增加
                        self.near_add_cd(open_user, map_cell_index as u32, buff_id, au);
                    } else if NEAR_SKILL_DAMAGE_PAIR == buff_function_id {
                        //相临都玩家造成技能伤害
                        self.near_skill_damage(open_user, map_cell_index as u32, buff_id, au);
                    }
                }
            }
        }
    }

    ///匹配buff
    pub fn trigger_open_map_cell_buff(
        &mut self,
        map_cell_index: usize,
        user_id: u32,
        au: &mut ActionUnitPt,
        is_pair: bool,
    ) {
        //匹配玩家身上的
        self.match_open_map_cell_buff_for_user(user_id, au, is_pair);

        //匹配地图快上的
        self.match_open_map_cell_buff_for_map_cell(user_id, map_cell_index, au, is_pair);
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
