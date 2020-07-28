use crate::entity::battle::{BattleCterState, TargetType};
use crate::TEMPLATES;
use log::warn;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use tools::protos::base::{BattleCharacterPt, CharacterPt};
use tools::templates::buff_temp::BuffTemp;
use tools::templates::character_temp::CharacterTemp;
use tools::templates::item_temp::ItemTemp;
use tools::templates::skill_temp::SkillTemp;

#[derive(Clone, Debug, Default)]
pub struct Character {
    pub user_id: u32, //玩家id
    pub cter_id: u32, //角色的配置id
    pub grade: u32,
    pub skills: Vec<u32>,          //玩家次角色所有已解锁的技能id,
    pub last_use_skills: Vec<u32>, //上次使用的技能
}

impl From<CharacterPt> for Character {
    fn from(cter_pt: CharacterPt) -> Self {
        let mut c = Character::default();
        c.cter_id = cter_pt.cter_id;
        c.grade = cter_pt.grade;
        c.skills = cter_pt.skills;
        c.last_use_skills = cter_pt.last_use_skills;
        c
    }
}

impl Into<CharacterPt> for Character {
    fn into(self) -> CharacterPt {
        let mut cter_pt = CharacterPt::new();
        cter_pt.set_cter_id(self.cter_id);
        cter_pt.set_grade(self.grade);
        cter_pt
    }
}

#[derive(Clone, Debug, Default)]
pub struct BattleCharacter {
    pub user_id: u32,                    //玩家id
    pub cter_id: u32,                    //角色的配置id
    pub atk: u32,                        //攻击力
    pub hp: i32,                         //角色血量
    pub defence: u32,                    //角色防御
    pub energy: u32,                     //角色能量
    pub element: u8,                     //角色元素
    pub cell_index: usize,               //角色所在位置
    pub skills: Vec<Skill>,              //玩家选择的主动技能id
    pub passive_buffs: Vec<Buff>,        //被动技能id
    pub target_id: u32,                  //玩家目标
    pub buff_array: Vec<Buff>,           //角色身上的buff
    pub state: u8,                       //角色状态
    pub residue_open_times: u8,          //剩余翻地图块次数
    pub turn_times: u32,                 //轮到自己的次数
    pub is_can_attack: bool,             //是否可以攻击
    pub items: HashMap<u32, Item>,       //角色身上的道具
    pub recently_open_cell_index: isize, //最近一次翻开的地图块
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: u32,                        //物品id
    pub skill_temp: &'static SkillTemp, //物品带的技能
}

#[derive(Clone, Debug)]
pub struct Skill {
    pub id: u32,
    pub skill_temp: &'static SkillTemp,
    pub cd_times: i8, //剩余cd,如果是消耗能量则无视这个值
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

#[derive(Clone, Debug)]
pub struct Buff {
    pub id: u32,
    pub buff_temp: &'static BuffTemp,
    pub trigger_timesed: i8, //已经触发过的次数
    pub keep_times: i8,      //持续轮数
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
        };
        b
    }
}

impl BattleCharacter {
    pub fn init(cter: &Character) -> anyhow::Result<Self> {
        let mut battle_cter = BattleCharacter::default();
        battle_cter.recently_open_cell_index = -1;
        let cter_id = cter.cter_id;
        battle_cter.user_id = cter.user_id;
        battle_cter.cter_id = cter_id;
        battle_cter.target_id = 0;
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
            let mut skill = Skill::from(skill_temp);
            battle_cter.skills.push(skill);
        }
        battle_cter.cell_index = 0;
        let cter_temp: Option<&CharacterTemp> =
            TEMPLATES.get_character_ref().get_temp_ref(&cter_id);
        if cter_temp.is_none() {
            let str = format!("cter temp is none for cter_id:{}!", cter_id);
            warn!("{:?}", str.as_str());
            anyhow::bail!(str)
        }
        let cter_temp = cter_temp.unwrap();
        //初始化战斗属性,这里需要根据占位进行buff加成，但buff还没设计完，先放在这儿
        battle_cter.hp = cter_temp.hp as i32;
        battle_cter.atk = cter_temp.attack;
        battle_cter.defence = cter_temp.defence;
        battle_cter.element = cter_temp.element;
        battle_cter.energy = cter_temp.energy;
        for buff_id in cter_temp.passive_buff.iter() {
            let buff_temp = buff_ref.temps.get(buff_id).unwrap();
            let mut buff = Buff::from(buff_temp);
            battle_cter.passive_buffs.push(buff);
        }
        Ok(battle_cter)
    }

    ///将自身转换成protobuf结构体
    pub fn convert_to_battle_cter(&self) -> BattleCharacterPt {
        let mut battle_cter_pt = BattleCharacterPt::new();
        battle_cter_pt.user_id = self.user_id;
        battle_cter_pt.cter_id = self.cter_id;
        battle_cter_pt.hp = self.hp as u32;
        battle_cter_pt.defence = self.defence;
        battle_cter_pt.atk = self.atk;
        let mut v = Vec::new();
        for buff in self.buff_array.iter() {
            v.push(buff.id);
        }
        battle_cter_pt.buffs = v;
        battle_cter_pt
    }
}
