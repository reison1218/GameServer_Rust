use crate::room::character::Character;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use tools::protos::base::MemberPt;
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
    NotReady = 0,
    Ready = 1,
}

#[derive(Clone, Debug, Default)]
pub struct Member {
    pub user_id: u32,                   //玩家id
    pub nick_name: String,              //玩家昵称
    pub grade: u8,                      //玩家grade
    pub state: u8,                      //玩家状态
    pub team_id: u8,                    //玩家所属队伍id
    pub is_robot: bool,                 //是否的机器人
    pub cters: HashMap<u32, Character>, //玩家拥有的角色数组
    pub chose_cter: Character,          //玩家已经选择的角色
    pub join_time: u64,                 //玩家进入房间的时间
}

impl Member {
    ///获得玩家id
    pub fn get_user_id(&self) -> u32 {
        self.user_id
    }
}

impl From<PlayerBattlePt> for Member {
    fn from(mut pbp: PlayerBattlePt) -> Self {
        let mut member = Member::default();
        member.nick_name = pbp.get_nick_name().to_owned();
        member.user_id = pbp.user_id;
        member.state = MemberState::NotReady as u8;
        member.grade = pbp.grade as u8;
        let mut cters = HashMap::new();
        let res = pbp.take_cters();

        let v = res.to_vec();
        for i in v {
            let mut cter = Character::from(i);
            cter.user_id = pbp.user_id;
            cters.insert(cter.cter_id, cter);
        }
        member.cters = cters;
        member
    }
}

impl Into<MemberPt> for Member {
    fn into(self) -> MemberPt {
        let mut mp = MemberPt::new();
        mp.user_id = self.get_user_id();
        mp.state = self.state as u32;
        mp.grade = self.grade as u32;
        mp.nick_name = self.nick_name.clone();
        mp.team_id = self.team_id as u32;
        mp.join_time = self.join_time;
        let cp = self.chose_cter.clone().into();
        mp.set_cter(cp);
        mp
    }
}
