use crate::battle::battle::SummaryUser;
use crate::battle::battle_enum::buff_type::{
    ATTACKED_ADD_ENERGY, CAN_NOT_MOVED, CHANGE_SKILL, DEFENSE_NEAR_MOVE_SKILL_DAMAGE, LOCKED,
    LOCK_SKILLS, TRANSFORM_BUFF, TRAP_ADD_BUFF, TRAP_SKILL_DAMAGE,
};
use crate::battle::battle_enum::skill_judge_type::{LIMIT_ROUND_TIMES, LIMIT_TURN_TIMES};
use crate::battle::battle_enum::skill_type::WATER_TURRET;
use crate::battle::battle_enum::EffectType::AddSkill;
use crate::battle::battle_enum::{ActionType, EffectType, TRIGGER_SCOPE_NEAR};
use crate::battle::battle_skill::Skill;
use crate::robot::robot_trigger::RobotTriggerType;
use crate::robot::RememberCell;
use crate::room::RoomType;
use crate::TEMPLATES;
use crate::{battle::battle::BattleData, room::map_data::MapCell};
use log::{error, warn};
use std::str::FromStr;
use tools::macros::GetMutRef;
use tools::protos::base::{ActionUnitPt, EffectPt, TargetPt, TriggerEffectPt};

use super::battle_enum::skill_type::SKILL_PAIR_LIMIT_DAMAGE;
use super::mission::{trigger_mission, MissionTriggerType};
use super::{battle_enum::buff_type::ATTACKED_SUB_CD, battle_player::BattlePlayer};

///触发事件trait
pub trait TriggerEvent {
    ///翻开地图块时候触发,主要触发buff和游戏机制上的东西
    fn open_map_cell_trigger(
        &mut self,
        user_id: u32,
        au: &mut ActionUnitPt,
        is_pair: bool,
    ) -> anyhow::Result<Option<(u32, ActionUnitPt)>>;

    ///看到地图块触发
    fn map_cell_trigger_for_robot(&self, index: usize, robot_trigger_type: RobotTriggerType);

    ///被移动前触发buff
    fn before_moved_trigger(&self, from_user: u32, target_user: u32) -> anyhow::Result<()>;

    ///移动位置后触发事件
    fn after_move_trigger(
        &mut self,
        battle_player: &mut BattlePlayer,
        index: isize,
        is_change_index_both: bool,
    ) -> (bool, Vec<(u32, ActionUnitPt)>);

    ///使用技能后触发
    fn after_use_skill_trigger(
        &mut self,
        user_id: u32,
        skill_id: u32,
        is_item: bool,
        au: &mut ActionUnitPt,
    );

    fn before_use_skill_trigger(&mut self, user_id: u32) -> anyhow::Result<()>;

    ///被攻击前触发
    fn attacked_before_trigger(&mut self, user_id: u32, target_pt: &mut TargetPt);

    ///被攻击后触发
    fn attacked_after_trigger(&mut self, user_id: u32, target_pt: &mut TargetPt);

    ///受到攻击伤害后触发
    fn attacked_hurted_trigger(&mut self, user_id: u32, target_pt: &mut TargetPt);

    ///地图刷新时候触发buff
    fn before_map_refresh_buff_trigger(&mut self);

    ///buff失效时候触发
    fn buff_lost_trigger(&mut self, user_id: u32, buff_id: u32);

    ///角色死亡触发
    fn after_cter_died_trigger(
        &mut self,
        from_user: u32,
        user_id: u32,
        is_last_one: bool,
        is_punishment: bool,
    );
}

impl BattleData {
    ///触发陷阱
    pub fn trigger_trap(
        &mut self,
        battle_player: &mut BattlePlayer,
        index: usize,
    ) -> Option<Vec<(u32, ActionUnitPt)>> {
        let map_cell = self.tile_map.map_cells.get_mut(index);
        if let None = map_cell {
            warn!("map do not has this map_cell!index:{}", index);
            return None;
        }
        let mut au_v = Vec::new();
        let turn_index = self.next_turn_index;
        let user_id = battle_player.get_user_id();
        let map_cell = map_cell.unwrap() as *mut MapCell;
        unsafe {
            for buff in map_cell.as_ref().unwrap().get_traps() {
                let buff_id = buff.get_id();
                let buff_function_id = buff.function_id;
                let mut target_pt = None;
                //判断是否是上buff的陷阱
                if TRAP_ADD_BUFF.contains(&buff_function_id) {
                    let buff_id = buff.buff_temp.par1;
                    let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
                    if let Err(e) = buff_temp {
                        warn!("{:?}", e);
                        continue;
                    }
                    battle_player
                        .cter
                        .add_buff(None, None, buff_id, Some(turn_index));

                    let mut target_pt_tmp = TargetPt::new();
                    target_pt_tmp
                        .target_value
                        .push(battle_player.get_map_cell_index() as u32);
                    target_pt_tmp.add_buffs.push(buff_id);
                    target_pt = Some(target_pt_tmp);
                } else if TRAP_SKILL_DAMAGE.contains(&buff_function_id) {
                    //造成技能伤害的陷阱
                    let skill_damage = buff.buff_temp.par1 as i16;
                    let mut target_pt_temp = self.new_target_pt(user_id).unwrap();
                    let res =
                        self.deduct_hp(0, user_id, Some(skill_damage), &mut target_pt_temp, true);
                    if let Err(e) = res {
                        error!("{:?}", e);
                        continue;
                    }
                    target_pt = Some(target_pt_temp);
                }

                if target_pt.is_none() {
                    continue;
                }
                if buff.from_user.is_none() {
                    continue;
                }
                let mut target_pt = target_pt.unwrap();
                let mut aup = ActionUnitPt::new();
                let lost_buff = self.consume_buff(buff_id, None, Some(index), false);
                if let Some(lost_buff) = lost_buff {
                    target_pt.lost_buffs.push(lost_buff);
                }
                aup.from_user = buff.from_user.unwrap();
                aup.action_type = ActionType::Buff as u32;
                aup.action_value.push(buff.get_id());
                aup.targets.push(target_pt);
                au_v.push((0, aup));
            }
        }

        Some(au_v)
    }
}

impl TriggerEvent for BattleData {
    fn open_map_cell_trigger(
        &mut self,
        user_id: u32,
        au: &mut ActionUnitPt,
        is_pair: bool,
    ) -> anyhow::Result<Option<(u32, ActionUnitPt)>> {
        // let battle_players = self.battle_player.borrow_mut() as *mut HashMap<u32, BattleCharacter>;
        let battle_player = self.battle_player.get_mut(&user_id).unwrap();
        let index = battle_player.get_map_cell_index();
        //匹配玩家身上的buff
        self.trigger_open_map_cell_buff(None, user_id, au, is_pair);
        //匹配地图块的buff
        self.trigger_open_map_cell_buff(Some(index), user_id, au, is_pair);
        let battle_player = self.battle_player.get_mut(&user_id).unwrap();
        let map_cell = self.tile_map.map_cells.get(index).unwrap();
        let element = map_cell.element as u32;
        //配对了加金币
        if is_pair {
            //把配对可用的技能加入
            let mut skill_function_id;
            let mut skill_id;
            for skill in battle_player.cter.skills.values() {
                skill_function_id = skill.function_id;
                skill_id = skill.id;
                if !SKILL_PAIR_LIMIT_DAMAGE.contains(&skill_function_id) {
                    continue;
                }
                battle_player.flow_data.pair_usable_skills.insert(skill_id);
            }
            //配对了奖励金币
            let res;
            let temp = crate::TEMPLATES
                .constant_temp_mgr()
                .temps
                .get("reward_gold_pair_cell");
            match temp {
                Some(temp) => {
                    let value = u32::from_str(temp.value.as_str());
                    match value {
                        Ok(value) => res = value,
                        Err(e) => {
                            error!("{:?}", e);
                            res = 2;
                        }
                    }
                }
                None => {
                    res = 2;
                }
            }
            battle_player.add_gold(res as i32);
            //触发翻地图块任务;触发获得金币;触发配对任务
            trigger_mission(
                self,
                user_id,
                vec![
                    (MissionTriggerType::Pair, 1),
                    (MissionTriggerType::GetGold, res as u16),
                ],
                (element, 0),
            );
        }
        Ok(None)
    }

    fn map_cell_trigger_for_robot(&self, index: usize, robot_trigger_type: RobotTriggerType) {
        let map_cell = self.tile_map.map_cells.get(index).unwrap();
        let rc = RememberCell::new(index, map_cell.id);
        for cter in self.battle_player.values() {
            //如果不是机器人就continue；
            if !cter.is_robot() {
                continue;
            }
            cter.robot_data
                .as_ref()
                .unwrap()
                .trigger(rc.clone(), robot_trigger_type);
        }
    }

    fn before_moved_trigger(&self, from_user: u32, target_user: u32) -> anyhow::Result<()> {
        //先判断目标位置的角色是否有不动泰山被动技能
        let target_cter = self.get_battle_player(Some(target_user), true).unwrap();
        let mut buff_function_id;
        for buff in target_cter.cter.battle_buffs.buffs().values() {
            buff_function_id = buff.function_id;
            if buff_function_id == CAN_NOT_MOVED && from_user != target_user {
                anyhow::bail!(
                    "this cter can not be move!cter_id:{},buff_id:{}",
                    target_user,
                    CAN_NOT_MOVED
                )
            }
        }

        Ok(())
    }

    ///移动后触发事件，大多数为buff
    fn after_move_trigger(
        &mut self,
        battle_player: &mut BattlePlayer,
        index: isize,
        is_change_index_both: bool,
    ) -> (bool, Vec<(u32, ActionUnitPt)>) {
        let mut v = Vec::new();
        let self_mut = self.get_mut_ref();
        //触发陷阱
        let res = self_mut.trigger_trap(battle_player, index as usize);
        if let Some(res) = res {
            v.extend_from_slice(res.as_slice());
        }
        let mut is_died = false;
        let mut buff_function_id;
        let mut buff_id;
        let target_user = battle_player.user_id;
        let mut from_user;
        //触发别人的范围
        for other_player in self.battle_player.values() {
            if other_player.is_died() {
                continue;
            }
            let cter_index = other_player.get_map_cell_index() as isize;
            from_user = other_player.user_id;
            //踩到别人到范围
            for buff in other_player.cter.battle_buffs.buffs().values() {
                buff_function_id = buff.function_id;
                buff_id = buff.get_id();
                if !DEFENSE_NEAR_MOVE_SKILL_DAMAGE.contains(&buff_function_id) {
                    continue;
                }
                //换位置不触发"DEFENSE_NEAR_MOVE_SKILL_DAMAGE"
                if is_change_index_both {
                    continue;
                }
                for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                    let res = cter_index + scope_index;
                    if index != res {
                        continue;
                    }

                    unsafe {
                        let mut target_pt = self.new_target_pt(target_user).unwrap();
                        let res = self_mut.deduct_hp(
                            from_user,
                            target_user,
                            Some(buff.buff_temp.par1 as i16),
                            &mut target_pt,
                            true,
                        );
                        if let Err(e) = res {
                            error!("{:?}", e);
                        }

                        let mut other_aupt = ActionUnitPt::new();
                        other_aupt.from_user = other_player.user_id;
                        other_aupt.action_type = ActionType::Buff as u32;
                        other_aupt.action_value.push(buff_id);
                        other_aupt.targets.push(target_pt);
                        v.push((0, other_aupt));
                        break;
                    }
                }
                if battle_player.is_died() {
                    is_died = true;
                    break;
                }
            }
            //别人进入自己的范围触发
            //现在没有种buff，先注释代码
            // if battle_player.get_user_id() == other_cter.get_user_id() {
            //     continue;
            // }
            // for buff in battle_player.buffs.values_mut() {
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
            //             battle_player.get_user_id(),
            //             other_cter.get_user_id(),
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
        (is_died, v)
    }

    ///使用技能后触发
    fn after_use_skill_trigger(
        &mut self,
        user_id: u32,
        skill_id: u32,
        is_item: bool,
        au: &mut ActionUnitPt,
    ) {
        let battle_player = self.get_battle_player_mut(Some(user_id), true);
        if let Err(e) = battle_player {
            error!("{:?}", e);
            return;
        }
        let battle_player = battle_player.unwrap();

        //添加技能限制条件
        let skill_temp = TEMPLATES.skill_temp_mgr().get_temp(&skill_id).unwrap();
        if skill_temp.skill_judge == LIMIT_TURN_TIMES as u16 {
            battle_player.flow_data.turn_limit_skills.push(skill_id);
        } else if skill_temp.skill_judge == LIMIT_ROUND_TIMES as u16 {
            battle_player.flow_data.round_limit_skills.push(skill_id);
        }

        //战斗角色身上的技能
        let mut skill_s;
        let mut skill = None;
        if is_item {
            let res = TEMPLATES.skill_temp_mgr().get_temp(&skill_id);
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            let skill_temp = res.unwrap();
            skill_s = Skill::from(skill_temp);
            skill = Some(&mut skill_s);
        } else {
            let res = battle_player.cter.skills.get_mut(&skill_id);
            if let Some(res) = res {
                skill = Some(res);
            }
        }
        if let Some(skill) = skill {
            let skill_function_id = skill.function_id;
            //替换技能,水炮
            if skill_function_id == WATER_TURRET {
                let skill_temp = TEMPLATES.skill_temp_mgr().get_temp(&skill.skill_temp.par2);
                battle_player.cter.skills.remove(&skill_id);
                if let Err(e) = skill_temp {
                    error!("{:?}", e);
                    return;
                }
                let st = skill_temp.unwrap();

                let mut target_pt = TargetPt::new();
                //封装角色位置
                target_pt
                    .target_value
                    .push(battle_player.get_map_cell_index() as u32);
                //封装丢失技能
                target_pt.lost_skills.push(skill_id);
                //封装增加的技能
                let mut ep = EffectPt::new();
                ep.effect_type = AddSkill.into_u32();
                ep.effect_value = st.id;
                target_pt.effects.push(ep);
                //将新技能封装到内存
                let skill = Skill::from(st);
                battle_player.cter.skills.insert(skill.id, skill);
                //将target封装到proto
                au.targets.push(target_pt);
            }

            //使用后删除可用状态
            if SKILL_PAIR_LIMIT_DAMAGE.contains(&skill_function_id) {
                battle_player.flow_data.pair_usable_skills.remove(&skill_id);
            }
        }

        //使用技能任务
        trigger_mission(
            self,
            user_id,
            vec![(MissionTriggerType::UseSkill, 1)],
            (0, 0),
        );
    }

    fn before_use_skill_trigger(&mut self, user_id: u32) -> anyhow::Result<()> {
        let battle_player = self.get_battle_player_mut(Some(user_id), true);
        if let Err(e) = battle_player {
            anyhow::bail!("{:?}", e)
        }
        let battle_player = battle_player.unwrap();
        let mut buff_function_id;
        for buff in battle_player.cter.battle_buffs.buffs().values() {
            buff_function_id = buff.function_id;
            if LOCK_SKILLS.contains(&buff_function_id) {
                anyhow::bail!(
                    "this cter can not use skill!was locked!cter's buff:{}",
                    buff.get_id()
                )
            }
        }
        Ok(())
    }

    ///被攻击前触发
    fn attacked_before_trigger(&mut self, _: u32, _: &mut TargetPt) {}

    ///被攻击后触发
    fn attacked_after_trigger(&mut self, user_id: u32, target_pt: &mut TargetPt) {
        let battle_player = self.get_battle_player_mut(Some(user_id), true);
        if let Err(e) = battle_player {
            warn!("{:?}", e);
            return;
        }
        let battle_player = battle_player.unwrap();
        let max_energy = battle_player.cter.base_attr.max_energy;
        let mut buff_function_id;
        let buff_ids: Vec<u32> = battle_player
            .cter
            .battle_buffs
            .buffs()
            .keys()
            .copied()
            .collect();
        for buff_id in buff_ids {
            let buff = battle_player
                .cter
                .battle_buffs
                .buffs()
                .get(&buff_id)
                .unwrap();
            let par1 = buff.buff_temp.par1;
            buff_function_id = buff.function_id;

            //被攻击增加能量
            if ATTACKED_ADD_ENERGY.contains(&buff_function_id) && max_energy > 0 {
                let mut tep = TriggerEffectPt::new();
                tep.set_field_type(EffectType::AddEnergy.into_u32());
                tep.set_buff_id(buff_id);
                tep.set_value(par1);
                battle_player.cter.add_energy(par1 as i8);
                target_pt.passiveEffect.push(tep);
            }

            //被攻击减技能cd
            if ATTACKED_SUB_CD == buff_function_id {
                let mut tep = TriggerEffectPt::new();
                tep.set_field_type(EffectType::SubSkillCd.into_u32());
                tep.set_buff_id(buff_id);
                tep.set_value(par1);
                battle_player.cter.sub_skill_cd(Some(par1 as i8));
                target_pt.passiveEffect.push(tep);
            }
        }
    }

    ///受到普通攻击触发的buff
    fn attacked_hurted_trigger(&mut self, user_id: u32, target_pt: &mut TargetPt) {
        let battle_data = self as *mut BattleData;
        let battle_player = self.get_battle_player_mut(Some(user_id), true);
        if let Err(e) = battle_player {
            warn!("{:?}", e);
            return;
        }
        let battle_player = battle_player.unwrap();
        let mut buff_id;
        let mut buff_function_id;
        for buff in battle_player.cter.battle_buffs.buffs().values() {
            buff_id = buff.get_id();
            buff_function_id = buff.function_id;

            //被攻击打断技能
            if CHANGE_SKILL.contains(&buff_function_id) {
                unsafe {
                    battle_data
                        .as_mut()
                        .unwrap()
                        .consume_buff(buff_id, Some(user_id), None, false);
                }
                target_pt.lost_buffs.push(buff_id);
            }
        }
    }

    fn before_map_refresh_buff_trigger(&mut self) {
        let mut buff_function_id;
        //如果存活玩家>=2并且地图未配对的数量<=2则刷新地图
        for map_cell in self.tile_map.map_cells.iter() {
            for buff in map_cell.buffs.values() {
                buff_function_id = buff.function_id;
                if buff_function_id != LOCKED {
                    continue;
                }
                let from_user = buff.from_user;
                if from_user.is_none() {
                    continue;
                }
                let from_user = from_user.unwrap();
                let from_skill = buff.from_skill.unwrap();
                let battle_player = self.battle_player.get_mut(&from_user);
                if battle_player.is_none() {
                    continue;
                }
                let battle_player = battle_player.unwrap();
                let skill = battle_player.cter.skills.get_mut(&from_skill);
                if skill.is_none() {
                    continue;
                }
                let skill = skill.unwrap();
                skill.is_active = false;
            }
        }
    }

    ///buff失效时候触发
    fn buff_lost_trigger(&mut self, user_id: u32, buff_id: u32) {
        let battle_player = self.get_battle_player_mut(Some(user_id), true);
        if let Err(e) = battle_player {
            error!("{:?}", e);
            return;
        }

        let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id).unwrap();
        let buff_function_id = buff_temp.function_id;
        let battle_player = battle_player.unwrap();
        //如果是变身buff,那就变回来
        if TRANSFORM_BUFF.contains(&buff_function_id) {
            battle_player.transform_back();
        }
    }

    fn after_cter_died_trigger(
        &mut self,
        from_user: u32,
        user_id: u32,
        is_last_one: bool,
        is_punishment: bool,
    ) {
        let battle_player = self.get_battle_player(Some(user_id), false);
        if let Err(e) = battle_player {
            warn!("{:?}", e);
            return;
        }
        let battle_player = battle_player.unwrap();
        let gold = battle_player.gold;
        let self_league = battle_player.league.get_league_id();
        let mut punishment_score = -50;
        let mut reward_score;
        let con_temp = crate::TEMPLATES
            .constant_temp_mgr()
            .temps
            .get("punishment_summary");
        if let Some(con_temp) = con_temp {
            let reward_score_temp = f64::from_str(con_temp.value.as_str());
            match reward_score_temp {
                Ok(reward_score_temp) => punishment_score = reward_score_temp as i32,
                Err(e) => warn!("{:?}", e),
            }
        }

        //如果是惩罚结算
        let player_count = self.get_alive_player_num() as i32;

        let mut sp = SummaryUser::default();
        sp.user_id = user_id;
        sp.name = battle_player.name.clone();
        sp.cter_id = battle_player.get_cter_id();
        sp.league = battle_player.league.clone();
        sp.grade = battle_player.grade;
        let rank_vec_temp = &mut self.summary_vec_temp;
        rank_vec_temp.push(sp);
        //判断是否需要排行,如果需要则从第最后
        if is_last_one {
            let index = player_count as usize;
            let res = self.summary_vec.get_mut(index);
            if let None = res {
                warn!(
                    "the rank_vec's len is {},but the index is {}",
                    self.summary_vec.len(),
                    index
                );
                return;
            }
            let rank_vec = res.unwrap();
            let count = rank_vec_temp.len();
            let summary_award_temp_mgr = crate::TEMPLATES.summary_award_temp_mgr();
            let con_temp_mgr = crate::TEMPLATES.constant_temp_mgr();
            let res = con_temp_mgr.temps.get("max_grade");
            let mut max_grade = 2;
            if self.room_type == RoomType::OneVOneVOneVOneMatch {
                match res {
                    None => {
                        warn!("max_grade config is None!");
                    }
                    Some(res) => {
                        max_grade = match u8::from_str(res.value.as_str()) {
                            Ok(res) => res,
                            Err(e) => {
                                warn!("{:?}", e);
                                max_grade
                            }
                        }
                    }
                }
            }

            for sp in rank_vec_temp.iter_mut() {
                sp.summary_rank = index as u8;
                //不是匹配房间不结算段位，积分
                if self.room_type != RoomType::OneVOneVOneVOneMatch {
                    continue;
                }
                //进行结算
                if is_punishment {
                    reward_score = punishment_score;
                    sp.grade -= 1;
                    if sp.grade <= 0 {
                        sp.grade = 1;
                    }
                } else {
                    //计算基础分
                    let mut rank = sp.summary_rank + 1;
                    if rank == 1 {
                        sp.grade += 1;
                        if sp.grade > 2 {
                            sp.grade = max_grade;
                        }
                    } else {
                        sp.grade -= 1;
                        if sp.grade <= 0 {
                            sp.grade = 1;
                        }
                    }
                    let mut base_score = 0;
                    for index in 0..count {
                        rank += index as u8;
                        let score_temp = summary_award_temp_mgr.get_score_by_rank(rank);
                        if let Err(e) = score_temp {
                            warn!("{:?}", e);
                            continue;
                        }
                        base_score += score_temp.unwrap();
                    }
                    base_score /= count as i16;
                    //计算浮动分
                    let mut average_league = 0;
                    let mut league_count = 0;
                    for (cter_id, league_id) in self.leave_map.iter() {
                        if *cter_id == user_id {
                            continue;
                        }
                        league_count += 1;
                        average_league += *league_id;
                    }
                    average_league /= league_count;
                    let mut unstable = 0;
                    if self_league >= average_league {
                        unstable = 0;
                    } else if average_league > self_league {
                        unstable = (average_league - self_league) * 10;
                    }
                    reward_score = (base_score + unstable as i16) as i32;
                }
                sp.reward_score = reward_score;
                let res = sp.league.update_score(reward_score);
                if res == 0 {
                    sp.reward_score = 0;
                }
            }
            rank_vec.extend_from_slice(&rank_vec_temp[..]);
            rank_vec_temp.clear();
        }
        let map_cell = self.tile_map.get_map_cell_mut_by_user_id(user_id);
        if let Some(map_cell) = map_cell {
            map_cell.user_id = 0;
        }
        //将死掉的角色的金币都给攻击方
        if from_user == 0 && from_user == user_id {
            return;
        }
        let from_cter = self.get_battle_player_mut(Some(from_user), true);
        if let Ok(from_cter) = from_cter {
            from_cter.add_gold(gold);
        }
    }
}
