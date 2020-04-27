use super::*;

#[derive(Clone, Debug, Default)]
pub struct Member {
    user_id: u32,
    target: Target,
}

impl Member {
    ///获得玩家id
    pub fn get_user_id(&self) -> u32 {
        self.user_id
    }
}

#[derive(Clone, Debug, Default)]
pub struct Target {
    team_id: u32,
    user_id: u32,
}
