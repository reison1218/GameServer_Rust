use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
};

use log::{error, info, warn};
use tools::{
    protos::base::{BattleCharacterPt, TargetPt},
    templates::character_temp::{CharacterTemp, TransformInheritType},
};

use crate::room::member::Member;
use crate::{battle::battle_enum::buff_type::SUB_MOVE_POINT, TEMPLATES};

use super::{
    battle::{DayNight, Item},
    battle_buff::Buff,
    battle_enum::{
        buff_type::{
            ADD_ATTACK, ADD_ATTACK_AND_AOE, ATTACKED_SUB_DAMAGE, CAN_NOT_MOVED, CHANGE_SKILL,
            CHARGE, DAY_SKILLS, GD_ATTACK_DAMAGE, NEAR_ATTACKED_DAMAGE_ZERO, NIGHT_SKILLS,
            STONE_BUFF, SUB_ATTACK_DAMAGE,
        },
        BattleCterState, FromType,
    },
    battle_player::{IndexData, TransformInheritValue},
    battle_skill::Skill,
};

///角色战斗基础属性
#[derive(Clone, Debug, Default)]
pub struct BaseAttr {
    pub cter_id: u32,      //角色id
    pub user_id: u32,      //所属的玩家id
    pub cter_temp_id: u32, //角色的配置id
    pub atk: u8,           //攻击力
    pub hp: i16,           //角色血量
    pub defence: u8,       //角色防御
    pub energy: u8,        //角色能量
    pub max_energy: u8,    //能量上限
    pub element: u8,       //角色元素
    pub item_max: u8,      //道具数量上限
    pub team_id: u8,       //队伍id
}

///角色战斗buff
#[derive(Clone, Debug, Default)]
pub struct BattleBuff {
    buffs: HashMap<u32, Buff>,           //角色身上的buff
    passive_buffs: HashMap<u32, Buff>,   //被动技能id
    add_damage_buffs: HashMap<u32, u32>, //伤害加深buff key:buffid value:叠加次数
    sub_damage_buffs: HashMap<u32, u32>, //减伤buff  key:buffid value:叠加次数
}

impl BattleBuff {
    pub fn init(&mut self, buff: Buff) {
        let buff_id = buff.get_id();
        let buff_function_id = buff.function_id;
        self.buffs.insert(buff.get_id(), buff.clone());
        self.passive_buffs.insert(buff.get_id(), buff.clone());
        if SUB_ATTACK_DAMAGE.contains(&buff_function_id) {
            self.add_sub_damage_buff(buff_id);
        }
        if ATTACKED_SUB_DAMAGE == buff_function_id {
            self.add_sub_damage_buff(buff_id);
        }
        if NEAR_ATTACKED_DAMAGE_ZERO == buff_function_id {
            self.add_sub_damage_buff(buff_id);
        }
    }

    pub fn add_add_damage_buffs(&mut self, buff_id: u32) {
        if !self.add_damage_buffs.contains_key(&buff_id) {
            self.add_damage_buffs.insert(buff_id, 1);
        } else {
            let &res = self.add_damage_buffs.get(&buff_id).unwrap();
            let res = res + 1;
            self.add_damage_buffs.insert(buff_id, res);
        }
    }

    pub fn add_sub_damage_buff(&mut self, buff_id: u32) {
        if !self.sub_damage_buffs.contains_key(&buff_id) {
            self.sub_damage_buffs.insert(buff_id, 1);
        } else {
            let &res = self.sub_damage_buffs.get(&buff_id).unwrap();
            let res = res + 1;
            self.sub_damage_buffs.insert(buff_id, res);
        }
    }

    pub fn add_buff_for_buffs(&mut self, buff: Buff) {
        self.buffs.insert(buff.get_id(), buff);
    }

    pub fn sub_damage_buffs(&self) -> &HashMap<u32, u32> {
        &self.sub_damage_buffs
    }

    pub fn buffs(&self) -> &HashMap<u32, Buff> {
        &self.buffs
    }

    pub fn buffs_mut(&mut self) -> &mut HashMap<u32, Buff> {
        &mut self.buffs
    }

    pub fn get_buff_mut(&mut self, buff_id: u32) -> Option<&mut Buff> {
        self.buffs.get_mut(&buff_id)
    }

    pub fn get_buff(&self, buff_id: u32) -> Option<&Buff> {
        self.buffs.get(&buff_id)
    }

    pub fn get_gd_buff(&mut self) -> Option<&mut Buff> {
        let mut buff_function_id;
        for buff in self.buffs.values_mut() {
            buff_function_id = buff.function_id;
            if buff_function_id == GD_ATTACK_DAMAGE[0] {
                return Some(buff);
            }
        }
        None
    }
}

///角色战斗数据
#[derive(Clone, Default)]
pub struct BattleCharacter {
    pub base_attr: BaseAttr,                               //基础属性
    pub battle_buffs: BattleBuff,                          //战斗buff
    pub index_data: IndexData,                             //角色位置数据
    pub state: BattleCterState,                            //角色状态
    pub revenge_user_id: u32,                              //复仇角色
    pub skills: HashMap<u32, Skill>,                       //玩家选择的主动技能id
    pub items: HashMap<u32, Item>,                         //角色身上的道具
    pub self_transform_cter: Option<Box<BattleCharacter>>, //自己变身的角色
    pub self_cter: Option<Box<BattleCharacter>>,           //原本的角色
    pub owner: Option<(u32, FromType)>,                    //主人id,来源类型
    pub minons: HashSet<u32>,                              //宠物id
    pub is_major: bool,                                    //是否是主角色
    pub day_night: DayNight,                               //当前角色日夜状态，用于切换技能和buff
}

tools::get_mut_ref!(BattleCharacter);

impl BattleCharacter {
    ///初始化战斗角色数据
    pub fn init(member: &Member, cter_id: u32) -> anyhow::Result<Self> {
        let cter = &member.chose_cter;
        let mut battle_cter = BattleCharacter::default();
        let cter_temp_id = cter.cter_temp_id;

        let skill_ref = TEMPLATES.skill_temp_mgr();
        let buff_ref = TEMPLATES.buff_temp_mgr();
        for skill_id in cter.skills.iter() {
            let res = skill_ref.temps.get(skill_id);
            if res.is_none() {
                let str = format!(
                    "there is no skill for skill_id:{}!cter_temp_id:{}",
                    skill_id, cter_temp_id
                );
                warn!("{:?}", str.as_str());
                anyhow::bail!(str)
            }
            let skill_temp = res.unwrap();
            let skill = Skill::from_skill_temp(skill_temp, true);
            battle_cter.skills.insert(*skill_id, skill);
        }
        let cter_temp = TEMPLATES.character_temp_mgr().get_temp_ref(&cter_temp_id);
        if cter_temp.is_none() {
            let str = format!("cter_temp is none for cter_id:{}!", cter_temp_id);
            warn!("{:?}", str.as_str());
            anyhow::bail!(str)
        }
        let cter_temp = cter_temp.unwrap();
        //初始化战斗属性,这里需要根据占位进行buff加成，但buff还没设计完，先放在这儿
        battle_cter.base_attr.user_id = member.user_id;
        battle_cter.base_attr.cter_id = cter_id;
        battle_cter.base_attr.cter_temp_id = cter_temp_id;
        battle_cter.base_attr.hp = cter_temp.hp;
        battle_cter.base_attr.atk = cter_temp.attack;
        battle_cter.base_attr.defence = cter_temp.defence;
        battle_cter.base_attr.element = cter_temp.element;
        battle_cter.base_attr.energy = cter_temp.start_energy;
        battle_cter.base_attr.max_energy = cter_temp.max_energy;
        battle_cter.base_attr.item_max = cter_temp.usable_item_count;
        battle_cter.base_attr.team_id = member.team_id;
        battle_cter.is_major = true;
        cter_temp.passive_buff.iter().for_each(|buff_id| {
            let buff_temp = buff_ref.temps.get(buff_id).unwrap();
            let buff = Buff::from(buff_temp);
            battle_cter.battle_buffs.init(buff);
        });
        Ok(battle_cter)
    }

    pub fn init_for_minon(
        user_id: u32,
        team_id: u8,
        from_cter_id: u32,
        from_type: FromType,
        cter_id: u32,
        cter_temp_id: u32,
        index: usize,
        turn_index: usize,
    ) -> anyhow::Result<Self> {
        let cter_temp = TEMPLATES.character_temp_mgr().get_temp_ref(&cter_temp_id);
        if cter_temp.is_none() {
            let str = format!("cter_temp is none for cter_id:{}!", cter_temp_id);
            warn!("{:?}", str.as_str());
            anyhow::bail!(str)
        }
        let cter_temp = cter_temp.unwrap();
        let mut battle_cter = BattleCharacter::default();

        let buff_ref = TEMPLATES.buff_temp_mgr();
        let skill_ref = TEMPLATES.skill_temp_mgr();
        for skill_group in cter_temp.skills.iter() {
            for skill_id in skill_group.group.iter() {
                let res = skill_ref.temps.get(skill_id);
                if res.is_none() {
                    let str = format!("there is no skill for skill_id:{}!", skill_id);
                    warn!("{:?}", str.as_str());
                    anyhow::bail!(str)
                }
                let skill_temp = res.unwrap();
                let skill = Skill::from_skill_temp(skill_temp, true);
                battle_cter.skills.insert(*skill_id, skill);
            }
        }

        //初始化战斗属性,这里需要根据占位进行buff加成，但buff还没设计完，先放在这儿
        battle_cter.base_attr.user_id = user_id;
        battle_cter.base_attr.cter_id = cter_id;
        battle_cter.base_attr.cter_temp_id = cter_temp_id;
        battle_cter.base_attr.hp = cter_temp.hp;
        battle_cter.base_attr.atk = cter_temp.attack;
        battle_cter.base_attr.defence = cter_temp.defence;
        battle_cter.base_attr.element = cter_temp.element;
        battle_cter.base_attr.energy = cter_temp.start_energy;
        battle_cter.base_attr.max_energy = cter_temp.max_energy;
        battle_cter.base_attr.item_max = cter_temp.usable_item_count;
        battle_cter.base_attr.team_id = team_id;
        battle_cter.is_major = false;
        battle_cter.owner = Some((from_cter_id, from_type));
        battle_cter.index_data.map_cell_index = Some(index);
        cter_temp.passive_buff.iter().for_each(|buff_id| {
            let buff_temp = buff_ref.temps.get(buff_id).unwrap();
            let mut buff = Buff::from(buff_temp);
            buff.turn_index = Some(turn_index);
            battle_cter.battle_buffs.init(buff);
        });
        Ok(battle_cter)
    }

    pub fn get_day_night_buff(&self, day_night: DayNight) -> Option<&Buff> {
        for buff in self.battle_buffs.buffs().values() {
            match day_night {
                DayNight::Day => {
                    if buff.function_id == DAY_SKILLS {
                        return Some(buff);
                    }
                }
                DayNight::Night => {
                    if buff.function_id == NIGHT_SKILLS {
                        return Some(buff);
                    }
                }
            }
        }
        None
    }

    pub fn can_be_move(&self) -> bool {
        let mut buff_function_id;
        for buff in self.battle_buffs.buffs().values() {
            buff_function_id = buff.function_id;
            if buff_function_id == CAN_NOT_MOVED {
                return false;
            }
        }
        true
    }

    pub fn get_stone_buff(&self) -> Option<&Buff> {
        let res = self
            .battle_buffs
            .buffs()
            .values()
            .find(|x| x.function_id == STONE_BUFF);
        res
    }

    pub fn get_cter_temp_id(&self) -> u32 {
        self.base_attr.cter_temp_id
    }

    pub fn get_cter_id(&self) -> u32 {
        self.base_attr.cter_id
    }
    pub fn get_user_id(&self) -> u32 {
        self.base_attr.user_id
    }

    ///加血
    pub fn add_hp(&mut self, hp: i16) -> bool {
        self.base_attr.hp += hp;
        if self.base_attr.hp <= 0 {
            let str = format!(
                "cter is died!because hp:{},cter_id:{},cter_temp_id:{},user_id:{},",
                hp,
                self.base_attr.cter_id,
                self.base_attr.cter_temp_id,
                self.get_user_id()
            );
            self.state = BattleCterState::Died;
            info!("{:?}", str);
        }
        self.state == BattleCterState::Died
    }

    pub fn is_died(&self) -> bool {
        self.state == BattleCterState::Died
    }

    pub fn is_has_add_attack_and_aoe(&self) -> bool {
        for (_, buff) in self.battle_buffs.buffs.iter() {
            if ADD_ATTACK_AND_AOE.contains(&buff.get_id()) {
                return true;
            }
        }
        false
    }

    ///从静态配置中初始化
    fn init_from_temp(&mut self, cter_temp: &CharacterTemp) {
        //先重制数据
        self.clean_all();
        //然后复制数据
        self.base_attr.cter_temp_id = cter_temp.id;
        self.base_attr.element = cter_temp.element;
        self.base_attr.hp = cter_temp.hp;
        self.base_attr.energy = cter_temp.start_energy;
        self.base_attr.max_energy = cter_temp.max_energy;
        self.base_attr.item_max = cter_temp.usable_item_count;
        self.base_attr.defence = cter_temp.defence;
        self.base_attr.atk = cter_temp.attack;
        self.state = BattleCterState::Alive;
        for skill_group in cter_temp.skills.iter() {
            for skill_id in skill_group.group.iter() {
                let skill_temp = TEMPLATES.skill_temp_mgr().get_temp(&skill_id);
                if let Err(e) = skill_temp {
                    warn!("{:?}", e);
                    continue;
                }
                let skill_temp = skill_temp.unwrap();
                let skill = Skill::from_skill_temp(skill_temp, true);
                self.skills.insert(skill.id, skill);
            }
        }
        cter_temp.passive_buff.iter().for_each(|buff_id| {
            let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(buff_id);
            if let Ok(buff_temp) = buff_temp {
                let buff = Buff::from(buff_temp);
                self.battle_buffs.init(buff);
            }
        });
    }

    pub fn clean_skill_cd(&mut self) {
        self.skills
            .values_mut()
            .filter(|skill| !skill.is_active)
            .for_each(|x| x.clean_cd())
    }

    pub fn sub_skill_cd(&mut self, value: Option<i8>) {
        let res;
        match value {
            Some(value) => {
                if value < 0 {
                    res = value;
                } else {
                    res = value * -1;
                }
            }
            None => {
                res = -1;
            }
        }

        self.skills
            .values_mut()
            .filter(|skill| !skill.is_active)
            .for_each(|x| {
                x.add_cd(res);
            })
    }

    pub fn add_energy(&mut self, value: i8) {
        let v = self.base_attr.energy as i8;
        let max = self.base_attr.max_energy as i8;
        let res = v + value;
        if res < 0 {
            self.base_attr.energy = 0;
        } else {
            let result = res.min(max);
            self.base_attr.energy = result as u8;
        }
    }

    ///角色地图块下标是否有效
    pub fn map_cell_index_is_choiced(&self) -> bool {
        self.index_data.map_cell_index.is_some()
    }

    ///设置角色地图块位置
    pub fn set_map_cell_index(&mut self, index: usize) {
        self.index_data.map_cell_index = Some(index);
    }

    ///获得角色地图块位置
    pub fn get_map_cell_index(&self) -> usize {
        if self.index_data.map_cell_index.is_none() {
            return 100;
        }
        self.index_data.map_cell_index.unwrap()
    }

    ///添加道具
    pub fn add_item(&mut self, item_id: u32) -> anyhow::Result<()> {
        let item_temp = TEMPLATES.item_temp_mgr().get_temp(&item_id)?;
        let skill_id = item_temp.trigger_skill;
        let skill_temp = TEMPLATES.skill_temp_mgr().get_temp(&skill_id)?;
        let item = Item {
            id: item_id,
            skill_temp,
        };
        if self.items.len() as u8 >= self.base_attr.item_max {
            anyhow::bail!(
                "this cter's item is full!item_max:{}",
                self.base_attr.item_max
            )
        }
        self.items.insert(item.id, item);
        Ok(())
    }

    pub fn move_index(&mut self, index: usize) {
        self.index_data.last_map_cell_index = Some(self.index_data.map_cell_index.unwrap());
        self.index_data.map_cell_index = Some(index);
    }

    ///消耗buff,如果有buff被删除了，则返回some，否则范围none
    pub fn consume_buff(&mut self, buff_id: u32, is_turn_start: bool) {
        let buff = self.battle_buffs.buffs.get_mut(&buff_id);
        if let Some(buff) = buff {
            if is_turn_start {
                buff.sub_keep_times();
            } else {
                buff.sub_trigger_timesed();
            }
        }
    }

    ///计算攻击力
    pub fn calc_damage(&self) -> i16 {
        let mut damage = self.base_attr.atk;

        for (buff_id, &times) in self.battle_buffs.add_damage_buffs.iter() {
            let buff = self.battle_buffs.buffs.get(buff_id);
            if buff.is_none() {
                continue;
            }
            let buff = buff.unwrap();
            for _ in 0..times {
                if buff_id == &1001 {
                    damage += buff.buff_temp.par2 as u8;
                } else {
                    damage += buff.buff_temp.par1 as u8;
                }
            }
        }
        damage as i16
    }

    ///添加buff
    pub fn add_buff(
        &mut self,
        from_cter: Option<u32>,
        from_skill: Option<u32>,
        buff_id: u32,
        turn_index: Option<usize>,
    ) {
        let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();
        let buff_function_id = buff_temp.function_id;

        //增伤
        if ADD_ATTACK.contains(&buff_function_id) {
            self.trigger_add_damage_buff(buff_id);
        }
        //减伤
        if SUB_ATTACK_DAMAGE.contains(&buff_function_id) {
            self.trigger_sub_damage_buff(buff_id);
        }

        if !self.battle_buffs.buffs.contains_key(&buff_id) {
            let buff = Buff::new(buff_temp, turn_index, from_cter, from_skill);
            self.battle_buffs.add_buff_for_buffs(buff);
        }
    }

    pub fn clean_all(&mut self) {
        self.skills.clear();
        self.battle_buffs.buffs.clear();
        self.battle_buffs.passive_buffs.clear();
        self.items.clear();
        self.index_data.map_cell_index = None;
        self.base_attr.element = 0;
        self.battle_buffs.sub_damage_buffs.clear();
        self.battle_buffs.add_damage_buffs.clear();
        self.base_attr.hp = 0;
        self.base_attr.atk = 0;
        self.base_attr.defence = 0;
        self.state = BattleCterState::Alive;
    }

    ///移除buff
    pub fn remove_buff(&mut self, buff_id: u32) {
        self.battle_buffs.buffs.remove(&buff_id);
        self.battle_buffs.add_damage_buffs.remove(&buff_id);
        self.battle_buffs.sub_damage_buffs.remove(&buff_id);
    }

    ///移除加伤buff
    pub fn remove_damage_buff(&mut self, buff_id: u32) {
        self.battle_buffs.add_damage_buffs.remove(&buff_id);
    }

    ///触发增加伤害buff
    fn trigger_add_damage_buff(&mut self, buff_id: u32) {
        if buff_id == 0 {
            return;
        }
        self.battle_buffs.add_add_damage_buffs(buff_id);
    }

    ///触发减伤buff
    fn trigger_sub_damage_buff(&mut self, buff_id: u32) {
        if buff_id == 0 {
            return;
        }
        self.battle_buffs.add_sub_damage_buff(buff_id);
    }

    ///回合开始触发
    pub fn trigger_turn_start(&mut self) {
        let mut buff_function_id;
        for buff in self.battle_buffs.buffs.values() {
            buff_function_id = buff.function_id;
            match buff_function_id {
                CHANGE_SKILL | CHARGE => {
                    let skill_id = buff.buff_temp.par1;

                    let skill_temp = TEMPLATES.skill_temp_mgr().temps.get(&skill_id);
                    match skill_temp {
                        None => {
                            error!(
                                "trigger_turn_start the skill_temp can not find!skill_id:{}",
                                skill_id
                            );
                        }
                        Some(st) => {
                            let skill = Skill::from_skill_temp(st, true);
                            self.skills.remove(&buff.buff_temp.par2);
                            self.skills.insert(skill_id, skill);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    ///触发抵挡攻击伤害
    pub fn trigger_attack_damge_gd(&mut self) -> (u32, bool) {
        let gd_buff = self.battle_buffs.get_gd_buff();
        let mut buff_id = 0;
        let mut is_remove = false;
        if gd_buff.is_none() {
            return (buff_id, is_remove);
        }
        let gd_buff = gd_buff.unwrap();

        buff_id = gd_buff.get_id();
        self.consume_buff(buff_id, false);
        let gd_buff = self.battle_buffs.buffs.get_mut(&buff_id).unwrap();
        if gd_buff.trigger_timesed <= 0 || gd_buff.keep_times <= 0 {
            is_remove = true;
        }
        (buff_id, is_remove)
    }

    ///将自身转换成protobuf结构体
    pub fn convert_to_battle_cter_pt(&self) -> BattleCharacterPt {
        let mut battle_cter_pt = BattleCharacterPt::new();
        battle_cter_pt.user_id = self.base_attr.user_id;
        battle_cter_pt.cter_id = self.base_attr.cter_id;
        battle_cter_pt.cter_temp_id = self.base_attr.cter_temp_id;
        battle_cter_pt.atk = self.base_attr.atk as u32;
        battle_cter_pt.hp = self.base_attr.hp as u32;
        battle_cter_pt.defence = self.base_attr.defence.into();
        battle_cter_pt.energy = self.base_attr.energy as u32;
        battle_cter_pt.index = self.get_map_cell_index() as u32;
        battle_cter_pt.is_major = self.is_major;
        self.battle_buffs
            .buffs
            .values()
            .for_each(|buff| battle_cter_pt.buffs.push(buff.get_id()));
        self.skills
            .values()
            .for_each(|skill| battle_cter_pt.skills.push(skill.into()));
        self.items
            .keys()
            .for_each(|&item_id| battle_cter_pt.items.push(item_id));
        self.minons
            .iter()
            .for_each(|&cter_id| battle_cter_pt.minons.push(cter_id));

        battle_cter_pt
    }

    ///变回来
    pub fn transform_back(&mut self) -> (TargetPt, Vec<Buff>) {
        let clone;

        if self.self_transform_cter.is_some()
            && self.get_cter_temp_id()
                != self
                    .self_transform_cter
                    .as_ref()
                    .unwrap()
                    .base_attr
                    .cter_temp_id
        {
            clone = self.self_transform_cter.as_mut().unwrap().clone();
        } else {
            clone = self.self_cter.as_mut().unwrap().clone();
        }

        let mut other_buff = vec![];
        //拷贝其他状态buff
        for buff in self.battle_buffs.buffs.values() {
            if buff.from_cter.is_none() {
                continue;
            }
            if buff.from_skill.is_none() {
                continue;
            }
            other_buff.push(buff.clone());
        }

        //拷贝需要继承的属性
        let transform_att_inherits = transform_att_inherit_copy(self, clone.base_attr.cter_temp_id);
        //拷贝需要继承的buff
        let transform_buff_inherits = transform_buff_inherit_copy(self);

        //开始数据转换
        let _ = std::mem::replace(self, *clone);
        //处理保留数据
        self.transform_inherit(transform_att_inherits, transform_buff_inherits);

        let mut target_pt = TargetPt::new();
        let cter_pt = self.convert_to_battle_cter_pt();
        let index = self.get_map_cell_index() as u32;
        target_pt.target_value.push(index);
        target_pt.target_value.push(index);
        target_pt.set_transform_cter(cter_pt);
        (target_pt, other_buff)
    }

    ///变身
    pub fn transform(
        &mut self,
        from_cter: u32,
        from_skill: Option<u32>,
        cter_temp_id: u32,
        buff_id: u32,
        next_turn_index: Option<usize>,
    ) -> anyhow::Result<TargetPt> {
        let cter_temp = TEMPLATES.character_temp_mgr().get_temp_ref(&cter_temp_id);
        if cter_temp.is_none() {
            anyhow::bail!("cter_temp can not find!cter_id:{}", cter_temp_id)
        }
        let cter_temp = cter_temp.unwrap();
        //拷贝需要继承的属性
        let transform_inherits = transform_att_inherit_copy(self, cter_temp_id);

        //拷贝需要继承的buff
        let transform_buff_inherits = transform_buff_inherit_copy(self);

        //保存原本角色
        if self.self_cter.is_none() {
            self.self_cter = Some(Box::new(self.clone()));
        }

        //初始化数据成另外一个角色
        self.init_from_temp(cter_temp);

        //将继承属性给当前角色
        self.transform_inherit(transform_inherits, transform_buff_inherits);

        //给新变身加变身buff
        let buff_temp = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            warn!("{:?}", e);
            anyhow::bail!("")
        }
        let buff_temp = buff_temp.unwrap();
        self.add_buff(Some(from_cter), from_skill, buff_id, next_turn_index);

        //添加变身附带的攻击buff
        let attack_buff_id = buff_temp.par1;
        let attack_buff = TEMPLATES.buff_temp_mgr().get_temp(&attack_buff_id);
        if let Ok(attack_buff) = attack_buff {
            let attack_buff_function_id = attack_buff.function_id;
            if ADD_ATTACK.contains(&attack_buff_function_id) {
                let buff_from_cter = self.base_attr.cter_id;
                self.add_buff(Some(buff_from_cter), None, attack_buff_id, next_turn_index);
            }
        }

        //保存自己变身的角色
        if self.base_attr.cter_id == from_cter {
            //此处必须执行两遍，因为第一遍只是给self_transform_cter赋值一份拷贝，但self_transform_cter里面的self_transform_cter是None
            self.self_transform_cter = Some(Box::new(self.clone()));
            //第二遍给self_transform_cter里面的self_transform_cter赋值一份拷贝
            self.self_transform_cter = Some(Box::new(self.clone()));
        }
        let mut target_pt = TargetPt::new();
        target_pt
            .target_value
            .push(self.get_map_cell_index() as u32);
        let battle_cter_pt = self.convert_to_battle_cter_pt();
        target_pt.set_transform_cter(battle_cter_pt);

        Ok(target_pt)
    }
    ///处理变身继承
    pub fn transform_inherit(
        &mut self,
        transform_att_inherits: Vec<TransformInherit>,
        transform_buff_inherits: Vec<Buff>,
    ) {
        for ti in transform_att_inherits {
            let ti_type = ti.0;
            match ti_type {
                TransformInheritType::Hp => {
                    self.base_attr.hp = ti.1.as_usize().unwrap() as i16;
                }
                TransformInheritType::Attack => {
                    self.base_attr.atk = ti.1.as_usize().unwrap() as u8;
                }
                TransformInheritType::MapIndex => {
                    self.index_data.map_cell_index = Some(ti.1.as_usize().unwrap());
                }
                TransformInheritType::Energy => {
                    self.base_attr.energy = ti.1.as_usize().unwrap() as u8;
                }
                _ => {}
            }
        }
        for buff in transform_buff_inherits {
            self.battle_buffs.buffs.insert(buff.get_id(), buff);
        }
    }
}

pub struct TransformInherit(TransformInheritType, TransformInheritValue);

pub fn transform_att_inherit_copy(
    battle_cter: &BattleCharacter,
    target_cter_temp_id: u32,
) -> Vec<TransformInherit> {
    let target_cter_temp = crate::TEMPLATES
        .character_temp_mgr()
        .get_temp_ref(&target_cter_temp_id)
        .unwrap();
    let transform_inherit = target_cter_temp.transform_inherit.clone();
    let mut v = vec![];
    for &ti in transform_inherit.iter() {
        let ti_type = TransformInheritType::try_from(ti);
        if let Err(e) = ti_type {
            error!("{:?}", e);
            continue;
        }
        let ti_type = ti_type.unwrap();
        let res = match ti_type {
            TransformInheritType::Hp => {
                TransformInheritValue::Int(battle_cter.base_attr.hp as usize)
            }
            TransformInheritType::Attack => {
                TransformInheritValue::Int(battle_cter.base_attr.atk as usize)
            }
            TransformInheritType::Energy => {
                TransformInheritValue::Int(battle_cter.base_attr.energy as usize)
            }
            TransformInheritType::MapIndex => {
                TransformInheritValue::Int(battle_cter.get_map_cell_index())
            }
            _ => TransformInheritValue::None,
        };
        v.push(TransformInherit(ti_type, res));
    }

    v
}

pub fn transform_buff_inherit_copy(battle_cter: &mut BattleCharacter) -> Vec<Buff> {
    let mut v = vec![];
    let mut buff_function_id;
    let mut need_remove = vec![];
    for buff in battle_cter.battle_buffs.buffs().values() {
        buff_function_id = buff.function_id;
        if SUB_MOVE_POINT == buff_function_id {
            v.push(buff.clone());
            need_remove.push(buff.get_id());
        }
    }
    for buff_id in need_remove {
        battle_cter.remove_buff(buff_id);
    }
    v
}
