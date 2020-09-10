use crate::battle::battle::Item;
use crate::battle::battle_buff::Buff;
use crate::battle::battle_enum::buff_type::{
    ADD_ATTACK, CHANGE_SKILL, NEAR_SUB_ATTACK_DAMAGE, SUB_ATTACK_DAMAGE,
};
use crate::battle::battle_enum::skill_type::GD_ATTACK_DAMAGE;
use crate::battle::battle_enum::{BattleCterState, TURN_DEFAULT_OPEN_CELL_TIMES};
use crate::battle::battle_skill::Skill;
use crate::TEMPLATES;
use log::{error, warn};
use std::collections::HashMap;
use tools::protos::base::{BattleCharacterPt, CharacterPt, TargetPt};
use tools::templates::character_temp::CharacterTemp;

#[derive(Clone, Debug, Default)]
pub struct Character {
    pub user_id: u32, //玩家id
    pub cter_id: u32, //角色的配置id
    pub grade: u8,
    pub skills: Vec<u32>,          //玩家次角色所有已解锁的技能id,
    pub last_use_skills: Vec<u32>, //上次使用的技能
}

impl From<CharacterPt> for Character {
    fn from(cter_pt: CharacterPt) -> Self {
        let mut c = Character::default();
        c.cter_id = cter_pt.cter_id;
        c.grade = cter_pt.grade as u8;
        c.skills = cter_pt.skills;
        c.last_use_skills = cter_pt.last_use_skills;
        c
    }
}

impl Into<CharacterPt> for Character {
    fn into(self) -> CharacterPt {
        let mut cter_pt = CharacterPt::new();
        cter_pt.set_cter_id(self.cter_id);
        cter_pt.set_grade(self.grade as u32);
        cter_pt
    }
}

///角色战斗数据
#[derive(Clone, Debug, Default)]
pub struct BattleCharacter {
    pub user_id: u32,                                      //玩家id
    pub cter_id: u32,                                      //角色的配置id
    pub grade: u8,                                         //等级
    pub atk: u8,                                           //攻击力
    pub hp: i16,                                           //角色血量
    pub defence: u8,                                       //角色防御
    pub energy: u8,                                        //角色能量
    pub max_energy: u8,                                    //能量上限
    pub element: u8,                                       //角色元素
    cell_index: Option<usize>,                             //角色所在位置
    pub skills: HashMap<u32, Skill>,                       //玩家选择的主动技能id
    pub passive_buffs: HashMap<u32, Buff>,                 //被动技能id
    pub buffs: HashMap<u32, Buff>,                         //角色身上的buff
    pub state: BattleCterState,                            //角色状态
    pub residue_open_times: u8,                            //剩余翻地图块次数
    pub is_can_attack: bool,                               //是否可以攻击
    pub items: HashMap<u32, Item>,                         //角色身上的道具
    pub open_cell_vec: Vec<usize>,                         //最近一次turn翻过的地图块
    pub is_pair: bool,                                     //最近一次翻块是否匹配
    pub last_cell_index: Option<usize>,                    //上一次所在地图块位置
    pub hp_max: i16,                                       //血上限
    pub item_max: u8,                                      //道具数量上限
    pub add_damage_buffs: HashMap<u32, u8>,                //伤害加深buff key:buffid value:叠加次数
    pub sub_damage_buffs: HashMap<u32, u8>,                //减伤buff  key:buffid value:叠加次数
    pub is_attacked: bool,                                 //一轮有没有受到攻击伤害
    pub turn_limit_skills: Vec<u32>,                       //turn限制技能
    pub round_limit_skills: Vec<u32>,                      //round限制技能
    pub self_transform_cter: Option<Box<BattleCharacter>>, //自己变身的角色
    pub self_cter: Option<Box<BattleCharacter>>,           //原本的角色
}

impl BattleCharacter {
    ///从静态配置中初始化
    fn init_from_temp(&mut self, cter_temp: &CharacterTemp) {
        //先重制数据
        self.clean_all();
        //然后复制数据
        self.cter_id = cter_temp.id;
        self.element = cter_temp.element;
        self.grade = 1;
        self.hp = cter_temp.hp;
        self.hp_max = cter_temp.hp;
        self.energy = cter_temp.start_energy;
        self.max_energy = cter_temp.max_energy;
        self.item_max = cter_temp.usable_item_count;
        self.defence = cter_temp.defence;
        self.atk = cter_temp.attack;
        self.state = BattleCterState::Alive;

        for skill_group in cter_temp.skills.iter() {
            for skill_id in skill_group.group.iter() {
                let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id);
                if let Err(e) = skill_temp {
                    warn!("{:?}", e);
                    continue;
                }
                let skill_temp = skill_temp.unwrap();
                let skill = Skill::from(skill_temp);
                self.skills.insert(skill.id, skill);
            }
        }
        cter_temp.passive_buff.iter().for_each(|buff_id| {
            let buff_temp = TEMPLATES.get_buff_ref().get_temp(buff_id).unwrap();
            let buff = Buff::from(buff_temp);
            self.add_buff(Some(self.user_id), None, buff.id, None);
            self.passive_buffs.insert(*buff_id, buff);
        });
    }

    ///变回来
    pub fn transform_back(&mut self) -> TargetPt {
        let cter;
        let is_self_transform;
        if self.self_transform_cter.is_some() {
            cter = self.self_transform_cter.as_mut().unwrap();
            is_self_transform = true;
        } else {
            cter = self.self_cter.as_mut().unwrap();
            is_self_transform = false;
        }
        //然后复制数据
        self.user_id = cter.user_id;
        self.cter_id = cter.cter_id;
        self.element = cter.element;
        self.grade = cter.grade;
        self.hp_max = cter.hp_max;
        self.max_energy = cter.max_energy;
        self.item_max = cter.item_max;
        self.defence = cter.defence;
        self.atk = cter.atk;
        self.state = cter.state;
        self.is_attacked = cter.is_attacked;
        self.skills = cter.skills.clone();
        self.buffs = cter.buffs.clone();
        self.passive_buffs = cter.passive_buffs.clone();

        //如果是从自己变身的角色变回去，则清空自己变身角色
        if is_self_transform {
            self.self_transform_cter = None;
        } else {
            self.self_cter = None;
        }

        let mut target_pt = TargetPt::new();
        let cter_pt = self.convert_to_battle_cter();
        target_pt.set_transform_cter(cter_pt);
        target_pt
    }

    ///变身
    pub fn transform(
        &mut self,
        from_user: u32,
        cter_id: u32,
        buff_id: u32,
    ) -> anyhow::Result<TargetPt> {
        let cter_temp = TEMPLATES.get_character_ref().get_temp_ref(&cter_id);
        if cter_temp.is_none() {
            anyhow::bail!("cter_temp can not find!cter_id:{}", cter_id)
        }
        let cter_temp = cter_temp.unwrap();
        //需要继承的属性
        let residue_open_times = self.residue_open_times;
        let hp = self.hp;
        let is_can_attack = self.is_can_attack;
        let cell_index = self.cell_index;
        let energy = self.energy;

        //生命原始指针
        let self_ptr = self as *mut BattleCharacter;
        unsafe {
            //先克隆一份
            let cter_clone = self_ptr.as_ref().unwrap().clone();
            //保存原本角色
            if self.self_cter.is_none() {
                self.self_cter = Some(Box::new(cter_clone));
            }
            //初始化数据成另外一个角色
            self.init_from_temp(cter_temp);
            //将继承属性给当前角色
            self.residue_open_times = residue_open_times;
            self.hp = hp;
            self.is_can_attack = is_can_attack;
            self.cell_index = cell_index;
            self.energy = energy;

            //给新变身加变身buff
            let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id);
            if let Err(e) = buff_temp {
                warn!("{:?}", e);
                anyhow::bail!("")
            }
            let buff_temp = buff_temp.unwrap();
            let buff = Buff::from(buff_temp);
            self.buffs.insert(buff.id, buff);
            //保存自己变身的角色
            if self.user_id == from_user {
                self.self_transform_cter = Some(Box::new(self_ptr.as_ref().unwrap().clone()));
            }
        }
        let mut target_pt = TargetPt::new();
        let battle_cter_pt = self.convert_to_battle_cter();
        target_pt.set_transform_cter(battle_cter_pt);
        Ok(target_pt)
    }

    ///角色地图块下标是否有效
    pub fn cell_index_is_choiced(&self) -> bool {
        self.cell_index.is_some()
    }

    ///设置角色地图块位置
    pub fn set_cell_index(&mut self, index: usize) {
        self.cell_index = Some(index);
    }

    ///获得角色地图块位置
    pub fn get_cell_index(&self) -> usize {
        if self.cell_index.is_none() {
            error!(
                "this cter's cell_index is None!user_id:{},cter_id:{}",
                self.user_id, self.cter_id
            );
            return 100;
        }
        self.cell_index.unwrap()
    }

    ///添加道具
    pub fn add_item(&mut self, item_id: u32) -> anyhow::Result<()> {
        let item_temp = TEMPLATES.get_item_ref().get_temp(&item_id)?;
        let skill_id = item_temp.trigger_skill;
        let skill_temp = TEMPLATES.get_skill_ref().get_temp(&skill_id)?;
        let item = Item {
            id: item_id,
            skill_temp,
        };
        if self.items.len() as u8 >= self.item_max {
            anyhow::bail!("this cter's item is full!item_max:{}", self.item_max)
        }
        self.items.insert(item.id, item);
        Ok(())
    }

    pub fn move_index(&mut self, index: usize) {
        self.last_cell_index = Some(self.cell_index.unwrap());
        self.cell_index = Some(index);
    }

    ///消耗buff,如果有buff被删除了，则返回some，否则范围none
    pub fn consume_buff(&mut self, buff_id: u32, is_turn_start: bool) {
        let buff = self.buffs.get_mut(&buff_id).unwrap();
        if is_turn_start {
            buff.sub_keep_times();
        } else {
            buff.sub_trigger_timesed();
        }
    }

    ///重制角色数据
    pub fn round_reset(&mut self) {
        self.is_attacked = false;
        self.is_can_attack = false;
        self.cell_index = None;
        self.open_cell_vec.clear();
        self.last_cell_index = None;
        self.round_limit_skills.clear();
    }

    pub fn clean_all(&mut self) {
        self.turn_reset();
        self.round_reset();
        self.skills.clear();
        self.buffs.clear();
        self.passive_buffs.clear();
        self.items.clear();
        self.cell_index = None;
        self.element = 0;
        self.sub_damage_buffs.clear();
        self.add_damage_buffs.clear();
        self.self_cter = None;
        self.self_transform_cter = None;
        self.grade = 1;
        self.hp = 0;
        self.atk = 0;
        self.defence = 0;
        self.state = BattleCterState::Alive;
    }

    ///计算攻击力
    pub fn calc_damage(&self) -> i16 {
        let mut damage = self.atk;

        for (buff_id, times) in self.add_damage_buffs.iter() {
            let buff = self.buffs.get(buff_id);
            if buff.is_none() {
                continue;
            }
            let buff = buff.unwrap();
            for _ in 0..*times {
                if buff_id == &1001 {
                    damage += buff.buff_temp.par2 as u8;
                } else {
                    damage += buff.buff_temp.par1 as u8;
                }
            }
        }
        damage as i16
    }

    ///计算减伤
    pub fn calc_reduce_damage(&self, attack_is_near: bool) -> i16 {
        let mut value = self.defence;

        for (buff_id, times) in self.sub_damage_buffs.iter() {
            let buff = self.buffs.get(buff_id);
            if buff.is_none() {
                continue;
            }
            let buff = buff.unwrap();
            if buff.id == NEAR_SUB_ATTACK_DAMAGE && !attack_is_near {
                continue;
            }
            for _ in 0..*times {
                value += buff.buff_temp.par1 as u8;
            }
        }
        value as i16
    }

    ///添加buff
    pub fn add_buff(
        &mut self,
        from_user: Option<u32>,
        from_skill: Option<u32>,
        buff_id: u32,
        turn_index: Option<usize>,
    ) {
        let buff_temp = TEMPLATES.get_buff_ref().get_temp(&buff_id);
        if let Err(e) = buff_temp {
            error!("{:?}", e);
            return;
        }
        let buff_temp = buff_temp.unwrap();

        let buff = Buff::new(buff_temp, turn_index, from_user, from_skill);

        //增伤
        if ADD_ATTACK.contains(&buff_id) {
            self.trigger_add_damage_buff(buff_id);
        }
        //减伤
        if SUB_ATTACK_DAMAGE.contains(&buff_id) {
            self.trigger_sub_damage_buff(buff_id);
        }

        self.buffs.insert(buff.id, buff);
    }

    ///移除buff
    pub fn remove_buff(&mut self, buff_id: u32) {
        self.buffs.remove(&buff_id);
        self.add_damage_buffs.remove(&buff_id);
        self.sub_damage_buffs.remove(&buff_id);
    }

    ///触发增加伤害buff
    pub fn trigger_add_damage_buff(&mut self, buff_id: u32) {
        if !self.add_damage_buffs.contains_key(&buff_id) {
            self.add_damage_buffs.insert(buff_id, 1);
        } else {
            let res = self.add_damage_buffs.get(&buff_id).unwrap();
            let res = *res + 1;
            self.add_damage_buffs.insert(buff_id, res);
        }
    }

    ///触发减伤buff
    pub fn trigger_sub_damage_buff(&mut self, buff_id: u32) {
        if !self.sub_damage_buffs.contains_key(&buff_id) {
            self.sub_damage_buffs.insert(buff_id, 1);
        } else {
            let res = self.sub_damage_buffs.get(&buff_id).unwrap();
            let res = *res + 1;
            self.sub_damage_buffs.insert(buff_id, res);
        }
    }

    ///初始化战斗角色数据
    pub fn init(cter: &Character) -> anyhow::Result<Self> {
        let mut battle_cter = BattleCharacter::default();
        let cter_id = cter.cter_id;
        battle_cter.user_id = cter.user_id;
        battle_cter.cter_id = cter_id;
        battle_cter.grade = cter.grade;
        let skill_ref = TEMPLATES.get_skill_ref();
        let buff_ref = TEMPLATES.get_buff_ref();
        for skill_id in cter.skills.iter() {
            let res = skill_ref.temps.get(skill_id);
            if res.is_none() {
                let str = format!("there is no skill for skill_id:{}!", skill_id);
                warn!("{:?}", str.as_str());
                anyhow::bail!(str)
            }
            let skill_temp = res.unwrap();
            let skill = Skill::from(skill_temp);
            battle_cter.skills.insert(*skill_id, skill);
        }
        let cter_temp: Option<&CharacterTemp> =
            TEMPLATES.get_character_ref().get_temp_ref(&cter_id);
        if cter_temp.is_none() {
            let str = format!("cter temp is none for cter_id:{}!", cter_id);
            warn!("{:?}", str.as_str());
            anyhow::bail!(str)
        }
        let cter_temp = cter_temp.unwrap();
        //初始化战斗属性,这里需要根据占位进行buff加成，但buff还没设计完，先放在这儿
        battle_cter.hp = cter_temp.hp;
        battle_cter.atk = cter_temp.attack;
        battle_cter.defence = cter_temp.defence;
        battle_cter.element = cter_temp.element;
        battle_cter.energy = cter_temp.start_energy;
        battle_cter.max_energy = cter_temp.max_energy;
        battle_cter.hp_max = cter_temp.hp;
        battle_cter.item_max = cter_temp.usable_item_count;
        cter_temp.passive_buff.iter().for_each(|buff_id| {
            let buff_temp = buff_ref.temps.get(buff_id).unwrap();
            let buff = Buff::from(buff_temp);
            battle_cter.add_buff(Some(battle_cter.user_id), None, buff.id, None);
            battle_cter.passive_buffs.insert(*buff_id, buff);
        });

        battle_cter.reset_residue_open_times();
        Ok(battle_cter)
    }

    ///重制翻块次数
    pub fn reset_residue_open_times(&mut self) {
        self.residue_open_times = TURN_DEFAULT_OPEN_CELL_TIMES;
    }

    ///回合开始触发
    pub fn trigger_turn_start(&mut self) {
        for buff in self.buffs.values() {
            if CHANGE_SKILL.contains(&buff.id) {
                let skill_id = buff.buff_temp.par1;

                let skill_temp = TEMPLATES.get_skill_ref().temps.get(&skill_id);
                match skill_temp {
                    None => {
                        error!(
                            "trigger_turn_start the skill_temp can not find!skill_id:{}",
                            skill_id
                        );
                    }
                    Some(st) => {
                        let skill = Skill::from(st);
                        self.skills.remove(&buff.buff_temp.par2);
                        self.skills.insert(skill_id, skill);
                    }
                }
            }
        }
    }

    ///回合结算
    pub fn turn_reset(&mut self) {
        //回合开始触发buff
        self.trigger_turn_start();
        //重制剩余翻块地处
        self.reset_residue_open_times();
        //重制是否可以攻击
        self.is_can_attack = false;
        //重制匹配状态
        self.is_pair = false;
        //重制是否翻过地图块
        self.open_cell_vec.clear();
        //清空turn限制
        self.turn_limit_skills.clear();
    }

    ///触发抵挡攻击伤害
    pub fn trigger_attack_damge_gd(&mut self) -> (u32, bool) {
        let gd_buff = self.buffs.get_mut(&GD_ATTACK_DAMAGE[0]);
        let mut buff_id = 0;
        let mut is_remove = false;
        if gd_buff.is_none() {
            return (buff_id, is_remove);
        }
        let gd_buff = gd_buff.unwrap();

        buff_id = gd_buff.id;
        self.consume_buff(buff_id, false);
        let gd_buff = self.buffs.get_mut(&buff_id).unwrap();
        if gd_buff.trigger_timesed <= 0 || gd_buff.keep_times <= 0 {
            is_remove = true;
        }
        (buff_id, is_remove)
    }

    ///校验角色是否死亡
    pub fn is_died(&self) -> bool {
        self.state == BattleCterState::Die
    }

    ///扣血
    pub fn sub_hp(&mut self, hp: i16) -> bool {
        self.hp -= hp;
        if self.hp <= 0 {
            self.hp = 0;
            self.state = BattleCterState::Die;
        }
        self.state == BattleCterState::Die
    }

    ///加血
    pub fn add_hp(&mut self, hp: i16) {
        self.hp += hp;
        if self.hp > self.hp_max {
            self.hp = self.hp_max;
        }
    }

    ///将自身转换成protobuf结构体
    pub fn convert_to_battle_cter(&self) -> BattleCharacterPt {
        let mut battle_cter_pt = BattleCharacterPt::new();
        battle_cter_pt.user_id = self.user_id;
        battle_cter_pt.cter_id = self.cter_id;
        battle_cter_pt.hp = self.hp as u32;
        battle_cter_pt.defence = self.defence.into();
        battle_cter_pt.atk = self.atk as u32;
        self.buffs
            .values()
            .for_each(|buff| battle_cter_pt.buffs.push(buff.id));
        self.skills
            .keys()
            .for_each(|skill_id| battle_cter_pt.skills.push(*skill_id));
        self.items
            .keys()
            .for_each(|item_id| battle_cter_pt.items.push(*item_id));
        battle_cter_pt
    }
}
