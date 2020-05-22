use super::*;

#[derive(Clone, Debug)]
pub enum UserType {
    Robot = 0,
    Real = 1,
}

#[derive(Clone, Debug)]
pub enum MemberState {
    NotReady = 0,
    Ready = 1,
}

#[derive(Clone, Debug, Default)]
pub struct Member {
    pub user_id: u32,      //玩家id
    pub nick_name: String, //玩家昵称
    pub user_type: u8,     //玩家类型，分为真实玩家和机器人
    pub state: u8,         //玩家状态
    pub target: Target,    //玩家目标
}

impl Member {
    ///获得玩家id
    pub fn get_user_id(&self) -> u32 {
        self.user_id
    }
}

#[derive(Clone, Debug, Default)]
pub struct Target {
    pub team_id: u8,
    pub user_id: u32,
}
