use crate::entity::character::{BattleCharacter, Character};
use std::collections::HashMap;
use tools::protos::base::{BattleCharacterPt, MemberPt};
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
pub struct Member {
    pub user_id: u32,                   //玩家id
    pub nick_name: String,              //玩家昵称
    pub user_type: u8,                  //玩家类型，分为真实玩家和机器人
    pub state: u8,                      //玩家状态
    pub team_id: u8,                    //玩家所属队伍id
    pub cters: HashMap<u32, Character>, //玩家拥有的角色数组
    pub chose_cter: Character,          //玩家已经选择的角色
    pub battle_cter: BattleCharacter,   //进入战斗后的角色数据
    pub join_time: u64,                 //玩家进入房间的时间
}

impl Member {
    ///获得玩家id
    pub fn get_user_id(&self) -> u32 {
        self.user_id
    }

    pub fn convert_to_battle_cter(&self) -> BattleCharacterPt {
        let mut battle_cter_pt = BattleCharacterPt::new();
        battle_cter_pt.user_id = self.user_id;
        battle_cter_pt.cter_id = self.battle_cter.cter_id;
        battle_cter_pt.grade = self.battle_cter.grade;
        battle_cter_pt.nick_name = self.nick_name.clone();
        battle_cter_pt.skills = self.battle_cter.skills.clone();
        battle_cter_pt.hp = self.battle_cter.hp;
        battle_cter_pt.defence = self.battle_cter.defence;
        battle_cter_pt.atk = self.battle_cter.atk;
        battle_cter_pt.set_birth_index(self.battle_cter.cell_index);
        battle_cter_pt.set_action_order(0);
        battle_cter_pt
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
            let cter = Character::from(i);
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
        mp.nick_name = self.nick_name.clone();
        mp.team_id = self.team_id as u32;
        mp.join_time = self.join_time;
        let cp = self.chose_cter.clone().into();
        mp.set_cter(cp);
        mp
    }
}
