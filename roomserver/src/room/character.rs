use tools::protos::base::{CharacterPt, LeaguePt};

#[derive(Clone, Debug, Default)]
pub struct Character {
    pub user_id: u32,              //玩家id
    pub cter_id: u32,              //角色的配置id
    pub is_robot: bool,            //是否是机器人
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

///段位数据
#[derive(Clone, Debug)]
pub struct League {
    pub league_id: i8,    //段位id
    pub score: i32,       //段位积分
    pub league_time: i64, //进入段位时间
}

impl From<&LeaguePt> for League {
    fn from(lp: &LeaguePt) -> Self {
        let mut l = League::default();
        l.score = lp.get_league_score();
        l.league_time = lp.league_time;
        let league_id = lp.league_id as i8;
        l.league_id = league_id;
        l
    }
}

impl League {
    pub fn round_reset(&mut self) {
        let old_id = self.league_id;
        self.league_id -= 1;
        if self.league_id <= 0 {
            self.league_id = 0;
            self.league_time = 0;
        } else {
            let res = crate::TEMPLATES
                .get_league_temp_mgr_ref()
                .get_temp(&self.league_id)
                .unwrap();
            if old_id != self.league_id {
                self.score = res.score;
                self.league_time = 0;
            }
        }
    }

    pub fn into_pt(&self) -> LeaguePt {
        let mut lpt = LeaguePt::new();
        lpt.set_league_id(self.league_id as i32);
        lpt.set_league_score(self.score);
        lpt.set_league_time(self.league_time);
        lpt
    }

    pub fn update(&mut self, league_id: i8, league_score: i32, league_time: i64) {
        self.league_id = league_id;
        self.score = league_score;
        self.league_time = league_time;
    }
}

impl Default for League {
    fn default() -> Self {
        League {
            score: 0,
            league_time: 0,
            league_id: 0,
        }
    }
}
