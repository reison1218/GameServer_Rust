use crate::entity::member::Member;
use crate::entity::member::MemberState;
use std::collections::HashMap;
use tools::protos::base::{MemberPt, TeamPt};

#[derive(Clone, Debug, Default)]
pub struct Team {
    pub id: u8,
    pub members: HashMap<u32, Member>,
}

impl Team {
    ///判断是否存在该成员
    pub fn is_exist_member(&self, user_id: &u32) -> bool {
        let result = self.members.contains_key(user_id);
        result
    }

    ///检查准备状态
    pub fn check_ready(&self) -> bool {
        for member in self.members.values() {
            if member.state == MemberState::NotReady as u8 {
                return false;
            }
        }
        true
    }

    ///添加成员
    pub fn add_member(&mut self, member: Member) {
        self.members.insert(member.get_user_id(), member);
    }

    ///移除玩家
    pub fn remove_member(&mut self, user_id: &u32) -> Option<Member> {
        self.members.remove(user_id)
    }

    ///获取成员的可变指针
    pub fn get_member_mut(&mut self, user_id: &u32) -> Option<&mut Member> {
        self.members.get_mut(&user_id)
    }

    ///获得玩家数量
    pub fn get_member_count(&self) -> usize {
        self.members.len()
    }

    ///转换protobuf
    pub fn convert_to_pt(&self) -> TeamPt {
        let mut pt = TeamPt::new();
        pt.team_id = self.id as u32;
        let mut v = Vec::new();
        for (_, member) in self.members.iter() {
            let mut mp = MemberPt::new();
            mp.set_user_id(member.user_id);
            mp.set_nick_name(member.nick_name.clone());
            mp.set_state(member.state as u32);
            v.push(mp);
        }
        let res = protobuf::RepeatedField::from(v);
        pt.set_members(res);
        pt
    }
}
