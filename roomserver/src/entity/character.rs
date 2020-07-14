use crate::TEMPLATES;
use log::warn;
use tools::protos::base::{BattleCharacterPt, CharacterPt};
use tools::templates::character_temp::CharacterTemp;

#[derive(Clone, Debug, Default)]
pub struct Character {
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
        cter_pt.set_skills(self.skills);
        cter_pt.set_cter_id(self.cter_id);
        cter_pt.set_grade(self.grade);
        cter_pt.set_last_use_skills(self.last_use_skills);
        cter_pt
    }
}

#[derive(Clone, Debug, Default)]
pub struct BattleCharacter {
    pub user_id: u32,     //玩家id
    pub cter_id: u32,     //角色的配置id
    pub atk: u32,         //攻击力
    pub hp: u32,          //角色血量
    pub defence: u32,     //角色防御
    pub cell_index: u32,  //角色所在位置
    pub skills: Vec<u32>, //玩家次角色所有已解锁的技能id
    pub target_id: u32,   //玩家目标
    pub turn_order: u32,  //行动回合顺序
}

impl BattleCharacter {
    pub fn init(cter: &Character) -> anyhow::Result<Self> {
        let mut battle_cter = BattleCharacter::default();
        let cter_id = cter.cter_id;
        battle_cter.cter_id = cter_id;
        battle_cter.target_id = 0;
        battle_cter.skills = cter.skills.clone();
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
        battle_cter.hp = cter_temp.hp;
        battle_cter.atk = cter_temp.attack;
        battle_cter.defence = cter_temp.defence;
        Ok(battle_cter)
    }

    pub fn convert_to_battle_cter(&self) -> BattleCharacterPt {
        let mut battle_cter_pt = BattleCharacterPt::new();
        battle_cter_pt.user_id = self.user_id;
        battle_cter_pt.cter_id = self.cter_id;
        battle_cter_pt.hp = self.hp;
        battle_cter_pt.defence = self.defence;
        battle_cter_pt.atk = self.atk;
        battle_cter_pt.set_location_index(self.cell_index);
        battle_cter_pt.set_turn_order(self.turn_order);
        battle_cter_pt
    }
}
