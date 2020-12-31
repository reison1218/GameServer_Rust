use crate::room::character::{Character, League};
use tools::protos::base::MemberPt;

#[derive(Clone, Debug, Default)]
pub struct Member {
    pub user_id: u32,          //玩家id
    pub nick_name: String,     //玩家昵称
    pub grade: u8,             //玩家grade
    pub league: League,        //段位数据
    pub team_id: u8,           //玩家所属队伍id
    pub is_robot: bool,        //是否的机器人
    pub chose_cter: Character, //玩家已经选择的角色
    pub join_time: u64,        //玩家进入房间的时间
}

impl Member {
    ///获得玩家id
    pub fn get_user_id(&self) -> u32 {
        self.user_id
    }
}

impl From<&MemberPt> for Member {
    fn from(mp: &MemberPt) -> Self {
        let mut m = Member::default();
        m.user_id = mp.user_id;
        let mut league = League::default();
        let res = crate::TEMPLATES
            .get_league_temp_mgr_ref()
            .get_temp(&(mp.league_id as u8));
        league.score = mp.league_score as i32;
        league.league_temp = res.unwrap();
        m.league = league;
        m.grade = mp.grade as u8;
        m.nick_name = mp.nick_name.clone();
        m.join_time = mp.join_time;
        m.team_id = mp.team_id as u8;
        m.chose_cter = Character::from(mp.cter.as_ref().unwrap());
        m
    }
}
