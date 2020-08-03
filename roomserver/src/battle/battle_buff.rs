use crate::battle::battle::{BattleData, Direction, Item};
use crate::battle::battle_enum::{ActionType, BattleCterState, EffectType};
use crate::battle::battle_enum::{TargetType, TRIGGER_SCOPE_NEAR};
use crate::room::character::BattleCharacter;
use crate::TEMPLATES;
use log::{error, info, warn};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use tools::protos::base::{ActionUnitPt, TargetPt};
use tools::templates::buff_temp::BuffTemp;

#[derive(Clone, Debug)]
pub struct Buff {
    pub id: u32,
    pub buff_temp: &'static BuffTemp,
    pub trigger_timesed: i8,   //已经触发过的次数
    pub keep_times: i8,        //持续轮数
    pub scope: Vec<Direction>, //buff的作用范围
}

impl Buff {
    pub fn get_target(&self) -> TargetType {
        let target_type = TargetType::from(self.buff_temp.target);
        target_type
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
    ///处理地图块配对成功之后的buff
    pub unsafe fn handler_cell_pair_buff(
        &mut self,
        user_id: u32,
        index: usize,
        au: &mut ActionUnitPt,
    ) -> anyhow::Result<Option<ActionUnitPt>> {
        let battle_cters = self.battle_cter.borrow_mut() as *mut HashMap<u32, BattleCharacter>;
        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let cell = self.tile_map.map.get(index).unwrap();
        let last_index = battle_cter.recently_open_cell_index as usize;
        let last_cell = self.tile_map.map.get(last_index).unwrap();
        let cell_temp = TEMPLATES.get_cell_ref().get_temp(&cell.id).unwrap();
        for buff in cell.buff.iter() {
            let mut target_pt = TargetPt::new();
            target_pt.target_type = TargetType::AnyPlayer as u32;
            target_pt.target_value = user_id;

            //获得道具
            if [30011, 30021, 30031, 30041].contains(&buff.id) {
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
                    let last_cell_user = battle_cters.as_mut().unwrap().get_mut(&last_cell.user_id);
                    if let Some(last_cell_user) = last_cell_user {
                        target_pt.target_value = last_cell_user.user_id;
                        au.targets.push(target_pt.clone());
                        last_cell_user.items.insert(item_id, item.clone());
                    }
                }
                target_pt.target_value = user_id;
                au.targets.push(target_pt);
                battle_cter.items.insert(item_id, item);
            } else if 30012 == buff.id {
                target_pt.effect_type = EffectType::Cure as u32;
                target_pt.effect_value = buff.buff_temp.par1;
                //配对恢复生命
                if buff.buff_temp.target == TargetType::CellPlayer as u32 {
                    let last_cell_user = battle_cters.as_mut().unwrap().get_mut(&last_cell.user_id);
                    if let Some(last_cell_user) = last_cell_user {
                        target_pt.target_value = last_cell_user.user_id;
                        au.targets.push(target_pt.clone());
                        last_cell_user.add_hp(buff.buff_temp.par1 as i32);
                    }
                }
                target_pt.target_value = user_id;
                au.targets.push(target_pt);
                //恢复生命值
                battle_cter.add_hp(buff.buff_temp.par1 as i32);
            //todo 通知客户端
            } else if 30022 == buff.id {
                //获得一个buff
                target_pt.buffs.push(buff.id);
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
                    let last_cell_user = battle_cters.as_mut().unwrap().get_mut(&last_cell.user_id);
                    if let Some(last_cell_user) = last_cell_user {
                        last_cell_user.buff_array.push(buff.clone());
                        target_pt.target_value = last_cell_user.user_id;
                        au.targets.push(target_pt.clone());
                    }
                }
                //给自己加
                target_pt.target_value = user_id;
                au.targets.push(target_pt);
                battle_cter.buff_array.push(buff);
            //todo 通知客户端
            } else if [30032].contains(&buff.id) {
                target_pt.effect_type = EffectType::AddSkillCd as u32;
                target_pt.effect_value = buff.buff_temp.par1;
                //相临的玩家技能cd增加
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
                        for skill in cter.skills.iter_mut() {
                            skill.add_cd(Some(buff.buff_temp.par1 as i8));
                        }
                    }
                    target_pt.target_value = cter.user_id;
                    au.targets.push(target_pt.clone());
                }
            //todo 通知客户端
            } else if [30042].contains(&buff.id) {
                target_pt.effect_type = EffectType::SkillDamage as u32;
                target_pt.effect_value = buff.buff_temp.par1;
                //相临都玩家造成技能伤害
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

                for user in v.iter() {
                    let cter = battle_cters.as_mut().unwrap().get_mut(user).unwrap();
                    target_pt.target_value = *user;
                    au.targets.push(target_pt.clone());
                    //造成技能伤害
                    let is_died = cter.sub_hp(buff.buff_temp.par1 as i32);
                    if is_died {
                        cter.state = BattleCterState::Die as u8;
                    }
                }
            //todo 通知客户端
            } else if [9].contains(&buff.id) {
                //处理世界块的逻辑
                //配对属性一样的地图块+hp
                //查看配对的cell的属性是否与角色属性匹配
                if cell_temp.element != battle_cter.element {
                    return Ok(None);
                }
                target_pt.effect_type = EffectType::Cure as u32;
                target_pt.effect_value = buff.buff_temp.par1;
                au.targets.push(target_pt);
                //获得buff
                battle_cter.add_hp(buff.buff_temp.par1 as i32);
            }
        }
        Ok(None)
    }

    ///处理地图块额外其他buff
    pub unsafe fn handler_cell_extra_buff(&mut self, user_id: u32, index: usize) {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;

        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();

        let cell = self.tile_map.map.get_mut(index).unwrap();

        for buff in cell.extra_buff.iter() {}
    }

    ///处理角色移动之后的事件
    pub unsafe fn handler_cter_move(&mut self, user_id: u32, index: usize) -> Vec<ActionUnitPt> {
        let mut v = Vec::new();
        let index = index as isize;
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
        let cter = self.battle_cter.get_mut(&user_id).unwrap();

        //踩到别人到范围
        for other_cter in battle_cters.as_mut().unwrap().values_mut() {
            let cter_index = other_cter.cell_index as isize;

            for buff in other_cter.buff_array.iter() {
                if buff.id != 1 {
                    continue;
                }
                for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                    let res = cter_index + scope_index;
                    if index != res {
                        continue;
                    }
                    cter.sub_hp(buff.buff_temp.par1 as i32);

                    let mut other_aupt = ActionUnitPt::new();
                    other_aupt.from_user = other_cter.user_id;
                    other_aupt.action_type = ActionType::Buff as u32;
                    other_aupt.action_value = buff.id;
                    let mut target_pt = TargetPt::new();
                    target_pt.target_type = TargetType::AnyPlayer as u32;
                    target_pt.target_value = cter.user_id;
                    target_pt.effect_type = EffectType::SkillDamage as u32;
                    target_pt.effect_value = buff.buff_temp.par1;
                    other_aupt.targets.push(target_pt);
                    v.push(other_aupt);
                    break;
                }
                if cter.is_died() {
                    //todo  处理角色死亡事件
                    break;
                }
            }
            //别人进入自己的范围触发
            if cter.user_id == other_cter.user_id {
                continue;
            }
            for buff in cter.buff_array.iter() {
                if buff.id != 1 {
                    continue;
                }
                for scope_index in TRIGGER_SCOPE_NEAR.iter() {
                    let res = index + scope_index;
                    if cter_index != res {
                        continue;
                    }
                    other_cter.sub_hp(buff.buff_temp.par1 as i32);

                    let mut self_aupt = ActionUnitPt::new();
                    self_aupt.from_user = user_id;
                    self_aupt.action_type = ActionType::Buff as u32;
                    self_aupt.action_value = buff.id;
                    let mut target_pt = TargetPt::new();
                    target_pt.target_type = TargetType::AnyPlayer as u32;
                    target_pt.target_value = other_cter.user_id;
                    target_pt.effect_type = EffectType::SkillDamage as u32;
                    target_pt.effect_value = buff.buff_temp.par1;
                    self_aupt.targets.push(target_pt);
                    v.push(self_aupt);
                    if other_cter.is_died() {
                        //todo  处理角色死亡事件
                        break;
                    }
                    break;
                }
            }
        }
        v
    }
}
