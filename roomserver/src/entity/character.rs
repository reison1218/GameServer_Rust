use crate::entity::battle::BattleCterState;
use crate::TEMPLATES;
use log::warn;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use tools::protos::base::{BattleCharacterPt, CharacterPt};
use tools::templates::character_temp::CharacterTemp;
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
    pub passive_skills: Vec<Skill>,      //被动技能id
    pub target_id: u32,                  //玩家目标
    pub turn_order: u32,                 //行动回合顺序
    pub buff_array: Vec<Skill>,          //角色身上的buff
    pub state: u8,                       //角色状态
    pub residue_open_times: u32,         //剩余翻地图块次数
    pub turn_times: u32,                 //轮到自己的次数
    pub is_can_attack: bool,             //是否可以攻击
    pub item_array: Vec<u32>,            //角色身上的道具
    pub recently_open_cell_index: usize, //最近一次翻开的地图块
}

#[derive(Clone, Debug, Default)]
pub struct Skill {
    pub id: u32,
    pub skill_temp: SkillTemp,
    pub cd_times: i8, //剩余cd,如果是消耗能量则无视这个值
}

impl BattleCharacter {
    pub fn init(cter: &Character) -> anyhow::Result<Self> {
        let mut battle_cter = BattleCharacter::default();
        let cter_id = cter.cter_id;
        battle_cter.user_id = cter.user_id;
        battle_cter.cter_id = cter_id;
        battle_cter.target_id = 0;
        let skill_ref = TEMPLATES.get_skill_ref();
        for skill_id in cter.skills.iter() {
            let res = skill_ref.temps.get(skill_id);
            if res.is_none() {
                let str = format!("there is no skill for skill_id:{}!", skill_id);
                warn!("{:?}", str.as_str());
                anyhow::bail!(str)
            }
            let skill_temp = res.unwrap();
            let mut skill = Skill::default();
            skill.id = skill_temp.id;
            skill.skill_temp = skill_temp.clone();
            skill.cd_times = 0;
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
        for skill_id in cter_temp.passive_skill.iter() {
            let skill_temp = skill_ref.temps.get(skill_id).unwrap();
            let mut skill = Skill::default();
            skill.id = skill_temp.id;
            skill.skill_temp = skill_temp.clone();
            battle_cter.passive_skills.push(skill);
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
        battle_cter_pt.set_index(self.cell_index as u32);
        battle_cter_pt.set_turn_order(self.turn_order);
        battle_cter_pt
    }
}
