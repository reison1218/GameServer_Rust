use crate::battle::battle::BattleData;
use crate::battle::battle_buff::Buff;
use crate::battle::battle_enum::{ActionType, TargetType};
use crate::handlers::battle_handler::Find;
use crate::room::character::BattleCharacter;
use crate::room::map_data::{Cell, CellType};
use crate::TEMPLATES;
use log::{error, info, warn};
use protobuf::Message;
use std::collections::HashMap;
use tools::cmd_code::ClientCode;
use tools::protos::base::{ActionUnitPt, TargetPt};
use tools::protos::battle::S_ACTION_NOTICE;
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
            cd_times: skill_temp.cd as i8,
            skill_temp: skill_temp,
        }
    }
}

impl BattleData {
    ///地图块换位置
    pub fn change_index(
        &mut self,
        user_id: u32,
        skill_id: u32,
        source_index: usize,
        target_index: usize,
    ) {
        let lock_skills = &TEMPLATES.get_skill_ref().lock_skills[..];
        let map_size = self.tile_map.map.len();
        //校验地图块
        if source_index > map_size || target_index > map_size {
            return;
        }
        let source_cell = self.tile_map.map.get(source_index).unwrap();
        let target_cell = self.tile_map.map.get(target_index).unwrap();

        //无效块不能换，锁定不能换
        if source_cell.id <= 1 || target_cell.id <= 1 {
            return;
        }
        //已配对的块不能换
        if source_cell.pair_index.is_some() || target_cell.pair_index.is_some() {
            return;
        }
        //锁定不能换
        for skill in source_cell.buff.iter() {
            if lock_skills.contains(&skill.id) {
                return;
            }
        }
        //锁定不能换
        for skill in target_cell.buff.iter() {
            if lock_skills.contains(&skill.id) {
                return;
            }
        }

        //先删掉
        let mut source_cell = self.tile_map.map.remove(source_index);
        let mut target_cell = self.tile_map.map.remove(target_index);

        //替换下标
        source_cell.index = target_index;
        target_cell.index = source_index;

        self.tile_map.map.insert(source_cell.index, source_cell);
        self.tile_map.map.insert(target_cell.index, target_cell);

        //通知客户端
        let mut au = ActionUnitPt::new();
        au.set_from_user(user_id);
        au.set_action_type(ActionType::Skill as u32);
        au.set_action_value(skill_id);
        let mut target_pt = TargetPt::new();
        target_pt.target_type = TargetType::Cell as u32;
        target_pt.target_value = source_index as u32;
        au.targets.push(target_pt);
        let mut v = Vec::new();
        v.push(au);
        self.push_action_notice(v);
    }

    ///展示地图块
    pub fn show_index(&mut self, user_id: u32, skill_id: u32, index: usize) {
        //校验index合法性
        let cell = self.tile_map.map.get(index);
        if cell.is_none() {
            let str = format!("show_index cell is none!index:{}", index);
            warn!("{:?}", str);
        }
        //校验index合法性
        let cell = cell.unwrap();
        let res = self.check_open_cell(cell);
        if let Err(e) = res {
            let str = format!("show_index {:?}", e);
            warn!("{:?}", str);
        }
        let cell_id = cell.id;
        //todo 下发给客户端
        let mut san = S_ACTION_NOTICE::new();
        let mut au = ActionUnitPt::new();
        au.action_type = ActionType::Skill as u32;
        au.action_value = skill_id;
        au.from_user = user_id;
        let mut target_pt = TargetPt::new();
        target_pt.target_type = TargetType::Cell as u32;
        au.targets.push(target_pt);
        san.action_uints.push(au);

        unsafe {
            let san = &mut san as *mut S_ACTION_NOTICE;
            let target_pt = san
                .as_mut()
                .unwrap()
                .action_uints
                .get_mut(0)
                .unwrap()
                .targets
                .get_mut(0)
                .unwrap();
            for member_id in self.battle_cter.clone().keys() {
                let bytes;
                if member_id == &user_id {
                    target_pt.target_value = cell_id;
                } else if target_pt.target_value > 0 {
                    target_pt.target_value = 0;
                }
                bytes = san.as_ref().unwrap().write_to_bytes().unwrap();
                self.send_2_client(ClientCode::ActionNotice, user_id, bytes);
            }
        }
    }

    ///移动玩家
    pub fn move_user(
        &mut self,
        user_id: u32,
        skill_id: u32,
        target_user: u32,
        target_index: usize,
    ) {
        //校验下标的地图块
        let target_cell = self.tile_map.map.get_mut(target_index);
        if let None = target_cell {
            warn!("there is no cell!index:{}", target_index);
            return;
        }
        let target_cell = target_cell.unwrap();
        //校验有效性
        if target_cell.id < CellType::Valid as u32 {
            warn!("this cell can not be choice!index:{}", target_index);
            return;
        }
        //校验世界块
        if target_cell.is_world {
            warn!("world cell can not be choice!index:{}", target_index);
            return;
        }

        target_cell.user_id = target_user;

        let target_cter = self.get_battle_cter_mut(Some(target_user));
        if let Err(e) = target_cter {
            warn!("{:?}", e);
            return;
        }

        //更新目标玩家的下标
        let target_cter = target_cter.unwrap();
        let last_index = target_cter.cell_index;
        target_cter.cell_index = target_index;
        //重制之前地图块上的玩家id
        let last_cell = self.tile_map.map.get_mut(last_index).unwrap();
        last_cell.user_id = 0;

        let mut au = ActionUnitPt::new();
        au.from_user = user_id;
        au.action_type = ActionType::Skill as u32;
        au.action_value = skill_id;

        let mut target_pt = TargetPt::new();
        target_pt.target_type = TargetType::AnyPlayer as u32;
        target_pt.target_value = target_user;

        //处理移动后事件
        unsafe {
            let v = self.handler_cter_move(target_user, target_index);
            self.push_action_notice(v);
        }
        //todo 通知客户的
    }

    ///技能伤害，并治疗
    pub unsafe fn skill_damage_and_cure(&mut self, user_id: u32, cter_index: usize, skill_id: u32) {
        let battle_cters = &mut self.battle_cter as *mut HashMap<u32, BattleCharacter>;
        let battle_cter = battle_cters.as_mut().unwrap().get_mut(&user_id).unwrap();
        let skill = battle_cter.skills.find(skill_id as usize).unwrap();
        let res = TEMPLATES
            .get_skill_scope_ref()
            .get_temp(&skill.skill_temp.scope);
        if let Err(e) = res {
            error!("{:?}", e);
            return;
        }
        let scope_temp = res.unwrap();
        let cter_index = cter_index as isize;
        let target_type = TargetType::from(skill.skill_temp.target);
        let res = self.cal_scope(user_id, cter_index, target_type, None, Some(scope_temp));
        if let Err(e) = res {
            error!("{:?}", e);
            return;
        }
        let v = res.unwrap();
        let mut add_hp = 0_u32;
        for user in v {
            let cter = self.get_battle_cter_mut(Some(user)).unwrap();
            add_hp += skill.skill_temp.par1;
            //扣血
            let is_died = cter.sub_hp(skill.skill_temp.par1 as i32);
            if is_died {
                //todo 触发角色死亡事件
            }
        }
        battle_cter.add_hp(add_hp as i32);
        //todo 通知客户端
    }

    ///自动配对地图块
    pub unsafe fn auto_pair_cell(&mut self, user_id: u32, skill_id: u32, target_index: usize) {
        let mut au = ActionUnitPt::new();
        let map = &mut self.tile_map.map as *mut Vec<Cell>;
        //校验目标下标的地图块
        let cell = map.as_mut().unwrap().get_mut(target_index);
        if let None = cell {
            warn!("there is no cell!index:{}", target_index);
            return;
        }
        let cell = cell.unwrap();
        //校验地图块
        let res = self.check_open_cell(cell);
        if let Err(e) = res {
            warn!("{:?}", e);
            return;
        }
        let battle_cter = self.get_battle_cter_mut(Some(user_id));
        if let Err(e) = battle_cter {
            error!("{:?}", e);
            return;
        }
        let battle_cter = battle_cter.unwrap();
        //找到与之匹配的地图块自动配对
        for _cell in map.as_mut().unwrap().iter_mut() {
            if _cell.id != cell.id {
                continue;
            }
            _cell.pair_index = Some(cell.index);
            cell.pair_index = Some(_cell.index);
        }
        //处理本turn不能攻击
        battle_cter.is_can_attack = false;
        //todo 通知客户端
    }

    ///减技能cd
    pub fn sub_cd(&mut self, user_id: u32, target_user: u32) {
        //目标的技能CD-2。
        let battle_cter = self.get_battle_cter_mut(Some(target_user));
        if let Err(e) = battle_cter {
            warn!("{:?}", e);
            return;
        }
        let battle_cter = battle_cter.unwrap();
        for _skill in battle_cter.skills.iter_mut() {
            _skill.sub_cd(Some(_skill.skill_temp.par1 as i8));
        }
        //todo 通知客户端
    }

    ///单体技能伤害
    pub fn single_skill_damage(&mut self, user_id: u32, skill_id: u32, target_user: u32) {
        let target_cter = self.get_battle_cter_mut(Some(target_user));
        if let Err(e) = target_cter {
            warn!("{:?}", e);
            return;
        }
        let target_cter = target_cter.unwrap();
        let skill = TEMPLATES.get_skill_ref().get_temp(&skill_id).unwrap();
        let is_died = target_cter.sub_hp(skill.par1 as i32);
        if is_died {
            //todo 触发角色死亡事件
        }
    }

    ///技能aoe伤害
    pub fn skill_aoe_damage(&mut self, user_id: u32, skill_id: u32, mut targets: Vec<u32>) {
        let battle_cter = self.get_battle_cter(Some(user_id)).unwrap();
        let skill = battle_cter.skills.find(skill_id as usize).unwrap();
        let damage = skill.skill_temp.par1 as i32;
        let damage_deep = skill.skill_temp.par2 as i32;
        let scope_id = skill.skill_temp.scope;
        let scope_temp = TEMPLATES.get_skill_scope_ref().get_temp(&scope_id);
        if let Err(e) = scope_temp {
            error!("{:?}", e);
            return;
        }
        let scope_temp = scope_temp.unwrap();

        //校验下标
        for index in targets.iter() {
            let cell = self.tile_map.map.get(*index as usize);
            if let None = cell {
                warn!("there is no cell!index:{}", index);
                return;
            }
        }

        let center_index = targets.remove(0) as isize;
        let target_type = TargetType::from(skill.skill_temp.target);

        //计算符合中心范围内的玩家
        let v = self
            .cal_scope(
                user_id,
                center_index,
                target_type,
                Some(targets),
                Some(scope_temp),
            )
            .unwrap();

        for member_id in v {
            let cter = self.get_battle_cter_mut(Some(member_id)).unwrap();
            let is_died;
            //判断是否中心位置
            if cter.cell_index == center_index as usize {
                is_died = cter.sub_hp(damage_deep);
            } else {
                is_died = cter.sub_hp(damage);
            }
            if is_died {
                //todo  触发角色死了的事件
            }
        }
    }

    ///上buff
    pub fn add_buff(&mut self, user_id: u32, skill_id: u32, target_array: Vec<u32>) {
        let mut v = Vec::new();
        //121, 211, 221, 311, 322, 20002
        let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id).unwrap();
        //先计算单体的
        let buff_id = skill_temp.buff as u32;
        let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id).unwrap();
        let buff = Buff::from(buff_temp);
        let buff_id = buff.id;
        let target_type = TargetType::from(skill_temp.target);

        let mut au = ActionUnitPt::new();
        au.from_user = user_id;
        au.action_type = ActionType::Skill as u32;
        au.action_value = skill_id;

        let mut target_pt = TargetPt::new();
        match target_type {
            TargetType::PlayerSelf => {
                let cter = self.get_battle_cter_mut(Some(user_id));
                if let Err(e) = cter {
                    warn!("{:?}", e);
                    return;
                }
                let cter = cter.unwrap();
                cter.buff_array.push(buff);

                target_pt.target_type = TargetType::PlayerSelf as u32;
                target_pt.target_value = user_id;
                target_pt.buffs.push(buff_id);
            }
            TargetType::UnPairNullCell => {
                let index = *target_array.get(0).unwrap() as usize;
                let cell = self.tile_map.map.get_mut(index);
                if cell.is_none() {
                    let str = format!("cell not find!index:{}", index);
                    warn!("{:?}", str);
                    return;
                }
                let cell = cell.unwrap();
                if cell.is_world {
                    let str = format!("world_cell can not be choice!index:{}", index);
                    warn!("{:?}", str);
                    return;
                }
                if cell.pair_index.is_some() {
                    let str = format!("this cell is already paired!index:{}", index);
                    warn!("{:?}", str);
                    return;
                }
                cell.extra_buff.push(buff);

                target_pt.target_type = TargetType::UnPairNullCell as u32;
                target_pt.target_value = index as u32;
                target_pt.buffs.push(buff_id);
            }
            _ => {}
        }
        au.targets.push(target_pt);
        v.push(au);
        self.push_action_notice(v);
    }
}
