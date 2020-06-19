use std::collections::HashMap;
use tools::protos::base::CharacterPt;
use tools::protos::server_protocol::PlayerBattlePt;

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
pub struct Charcter {
    pub temp_id: u32, //角色的配置id
    pub grade: u32,
    pub skills: Vec<u32>, //玩家次角色所有已解锁的技能id
}

impl From<CharacterPt> for Charcter {
    fn from(cter_pt: CharacterPt) -> Self {
        let mut c = Charcter::default();
        c.temp_id = cter_pt.temp_id;
        c.skills = cter_pt.skills;
        c
    }
}

#[derive(Clone, Debug, Default)]
pub struct BattleCharcter {
    pub temp_id: u32, //角色的配置id
    pub hp: u32,
    pub defence: u32,
    pub skills: Vec<u32>, //玩家次角色所有已解锁的技能id
    pub target_id: u32,   //玩家目标
}

#[derive(Clone, Debug, Default)]
pub struct Member {
    pub user_id: u32,                  //玩家id
    pub nick_name: String,             //玩家昵称
    pub user_type: u8,                 //玩家类型，分为真实玩家和机器人
    pub state: u8,                     //玩家状态
    pub team_id: u8,                   //玩家所属队伍id
    pub cters: HashMap<u32, Charcter>, //玩家拥有的角色数组
    pub chose_cter: Charcter,          //玩家已经选择的角色
    pub battle_cter: BattleCharcter,   //进入战斗后的角色数据
    pub join_time: u64,                //玩家进入房间的时间
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
        member.user_type = UserType::Real as u8;

        let mut cters = HashMap::new();
        let res = pbp.take_cters();

        let v = res.to_vec();
        for i in v {
            let cter = Charcter::from(i);
            cters.insert(cter.temp_id, cter);
        }
        member.cters = cters;
        member
    }
}
