use crate::room::character::{Character, League};
use std::collections::HashMap;
use tools::protos::base::{MemberPt, PunishMatchPt};

#[derive(Clone, Debug, Default)]
pub struct Member {
    pub user_id: u32,                   //玩家id
    pub nick_name: String,              //玩家昵称
    pub grade: u8,                      //玩家grade
    pub league: League,                 //段位数据
    pub team_id: u8,                    //玩家所属队伍id
    pub is_robot: bool,                 //是否的机器人
    pub cters: HashMap<u32, Character>, //玩家拥有的角色数组
    pub chose_cter: Character,          //玩家已经选择的角色
    pub punish_match: PunishMatch,      //匹配惩罚数据
    pub join_time: u64,                 //玩家进入房间的时间
}

impl From<&MemberPt> for Member {
    fn from(mp: &MemberPt) -> Self {
        let mut m = Member::default();
        m.user_id = mp.user_id;
        let league = League::from(mp.get_league());
        m.league = league;
        m.grade = mp.grade as u8;
        m.nick_name = mp.nick_name.clone();
        m.join_time = mp.join_time;
        m.team_id = mp.team_id as u8;
        m.chose_cter = Character::from(mp.cter.as_ref().unwrap());
        m.is_robot = mp.is_robot;
        m
    }
}

///匹配惩罚数据
#[derive(Debug, Clone, Copy, Default)]
pub struct PunishMatch {
    pub start_time: i64, //开始惩罚时间
    pub punish_id: u8,   //惩罚id
}

impl PunishMatch {
    pub fn add_punish(&mut self) {
        self.start_time = chrono::Local::now().timestamp_millis();
        self.punish_id += 1;
    }
}

impl Into<PunishMatchPt> for PunishMatch {
    fn into(self) -> PunishMatchPt {
        let mut pmp = PunishMatchPt::new();
        pmp.punish_id = self.punish_id as u32;
        pmp.start_time = self.start_time;
        pmp
    }
}

impl From<&PunishMatchPt> for PunishMatch {
    fn from(pmp: &PunishMatchPt) -> Self {
        let mut pm = PunishMatch::default();
        pm.punish_id = pmp.punish_id as u8;
        pm.start_time = pmp.start_time;
        pm
    }
}
