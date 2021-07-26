use crate::mgr::League;
use crate::room::character::Character;
use log::warn;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use tools::protos::base::{MemberPt, PunishMatchPt};
use tools::protos::server_protocol::PlayerBattlePt;

#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum UserType {
    Robot = 0,
    Real = 1,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum MemberState {
    AwaitConfirm = 0,
    NotReady = 1,
    Ready = 2,
}

impl Default for MemberState {
    fn default() -> Self {
        MemberState::NotReady
    }
}

impl MemberState {
    pub fn into_u8(self) -> u8 {
        let res: u8 = self.into();
        res
    }
}

#[derive(Clone, Debug, Default)]
pub struct Member {
    pub user_id: u32,                   //玩家id
    pub nick_name: String,              //玩家昵称
    pub grade: u8,                      //玩家grade
    pub grade_frame: u32,               //玩家grade相框
    pub soul: u32,                      //灵魂
    pub league: League,                 //段位数据
    pub state: MemberState,             //玩家状态
    pub team_id: u8,                    //玩家所属队伍id
    pub robot_temp_id: u32,             //是否的机器人,配置id
    pub cters: HashMap<u32, Character>, //玩家拥有的角色数组
    pub chose_cter: Character,          //玩家已经选择的角色
    pub punish_match: PunishMatch,      //匹配惩罚数据
    pub join_time: u64,                 //玩家进入房间的时间
}

impl Member {
    ///获得玩家id
    pub fn get_user_id(&self) -> u32 {
        self.user_id
    }

    ///处理匹配惩罚
    pub fn reset_punish_match(&mut self) -> Option<PunishMatch> {
        //先判断是否需要重制
        let start_time = self.punish_match.start_time;
        let id = self.punish_match.punish_id as u32;
        if id == 0 {
            return None;
        }
        let punish_temp = crate::TEMPLATES.punish_temp_mgr().get_temp(&id);
        if let Err(e) = punish_temp {
            warn!("{:?}", e);
            return None;
        }
        let punish_temp = punish_temp.unwrap();
        let end_time = start_time + punish_temp.punish_time;
        //处理跨天清0
        let is_today = tools::util::is_today(start_time);
        if !is_today && start_time > 0 {
            self.punish_match.reset(true);
            return Some(self.punish_match);
        }
        //处理过期
        let now_time = chrono::Local::now().timestamp_millis();
        if now_time >= end_time {
            self.punish_match.reset(false);
            return Some(self.punish_match);
        }
        None
    }
}

impl From<&PlayerBattlePt> for Member {
    fn from(pbp: &PlayerBattlePt) -> Self {
        let mut member = Member::default();
        member.nick_name = pbp.get_nick_name().to_owned();
        member.user_id = pbp.user_id;
        member.state = MemberState::NotReady;
        member.grade = pbp.grade as u8;
        member.grade_frame = pbp.grade_frame;
        member.soul = pbp.soul;

        let league = League::from(pbp.get_league());
        member.league = league;
        let mut cters = HashMap::new();
        let res = pbp.cters.clone();

        let v = res.to_vec();
        for i in v {
            let mut cter = Character::from(i);
            cter.user_id = pbp.user_id;
            cters.insert(cter.cter_temp_id, cter);
        }
        member.cters = cters;
        member.punish_match = PunishMatch::from(pbp.get_punish_match());
        member
    }
}

impl Into<MemberPt> for &Member {
    fn into(self) -> MemberPt {
        let mut mp = MemberPt::new();
        mp.user_id = self.get_user_id();
        mp.state = self.state as u32;
        mp.grade = self.grade as u32;
        mp.grade_frame = self.grade_frame;
        mp.soul = self.soul;
        mp.nick_name = self.nick_name.clone();
        mp.team_id = self.team_id as u32;
        mp.join_time = self.join_time;
        mp.set_league(self.league.into_pt());
        mp.robot_temp_id = self.robot_temp_id;
        let cp = self.chose_cter.clone().into();
        mp.set_cter(cp);
        mp
    }
}

///匹配惩罚数据
#[derive(Debug, Clone, Copy, Default)]
pub struct PunishMatch {
    pub start_time: i64, //开始惩罚时间
    pub punish_id: u8,   //惩罚id
    pub today_id: u8,
}

impl PunishMatch {
    pub fn add_punish(&mut self) {
        self.start_time = chrono::Local::now().timestamp_millis();
        let max_id = crate::TEMPLATES.punish_temp_mgr().max_id as u8;
        self.today_id += 1;
        if self.today_id >= max_id {
            self.today_id = max_id;
        }
        self.punish_id = self.today_id;
    }
    pub fn reset(&mut self, is_reset: bool) {
        self.start_time = 0;
        self.punish_id = 0;
        if is_reset {
            self.today_id = 0;
        }
    }
}

impl Into<PunishMatchPt> for PunishMatch {
    fn into(self) -> PunishMatchPt {
        let mut pmp = PunishMatchPt::new();
        pmp.punish_id = self.punish_id as u32;
        pmp.start_time = self.start_time;
        pmp.today_id = self.today_id as u32;
        pmp
    }
}

impl From<&PunishMatchPt> for PunishMatch {
    fn from(pmp: &PunishMatchPt) -> Self {
        let mut pm = PunishMatch::default();
        pm.punish_id = pmp.punish_id as u8;
        pm.start_time = pmp.start_time;
        pm.today_id = pmp.today_id as u8;
        pm
    }
}
