use tools::protos::base::CharacterPt;

#[derive(Clone, Debug, Default)]
pub struct Character {
    pub user_id: u32,              //玩家id
    pub cter_id: u32,              //角色的配置id
    pub skills: Vec<u32>,          //玩家次角色所有已解锁的技能id,
    pub last_use_skills: Vec<u32>, //上次使用的技能
}

impl From<CharacterPt> for Character {
    fn from(cter_pt: CharacterPt) -> Self {
        let mut c = Character::default();
        c.cter_id = cter_pt.cter_id;
        c.skills = cter_pt.skills;
        c.last_use_skills = cter_pt.last_use_skills;
        c
    }
}

impl Into<CharacterPt> for Character {
    fn into(self) -> CharacterPt {
        let mut cter_pt = CharacterPt::new();
        cter_pt.set_cter_id(self.cter_id);
        cter_pt.set_skills(self.skills);
        cter_pt.set_last_use_skills(self.last_use_skills);
        cter_pt
    }
}
