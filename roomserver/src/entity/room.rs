use super::*;
use crate::entity::member::Member;
use crate::entity::team::Team;
use chrono::{DateTime, Local, Utc};
use std::collections::HashMap;
use std::sync::atomic::Ordering;

pub enum RoomState {
    Await = 0,   //等待
    Started = 1, //已经开始
}

///行动单位
#[derive(Clone, Debug, Copy, Default)]
pub struct ActionUnit {
    team_id: u32,
    user_id: u32,
}

///房间结构体，封装房间必要信息
#[derive(Clone, Debug)]
pub struct Room {
    id: u64,                       //房间id
    owner_id: u32,                 //房主id
    map_id: u32,                   //地图id
    player_team: HashMap<u32, u8>, //玩家对应的队伍
    teams: HashMap<u8, Team>,      //队伍map
    orders: Vec<ActionUnit>,       //action队列
    state: u8,                     //房间状态
    time: DateTime<Utc>,           //房间创建时间
}

impl Room {
    ///构建一个房间的结构体
    fn new(map_id: u32, owner_id: u32) -> Room {
        let id: u64 = crate::ROOM_ID.fetch_add(10, Ordering::Relaxed);
        let time = Utc::now();
        let teams: HashMap<u8, Team> = HashMap::new();
        let orders: Vec<ActionUnit> = Vec::new();
        let player_team: HashMap<u32, u8> = HashMap::new();
        Room {
            id,
            owner_id,
            map_id,
            player_team,
            teams,
            orders,
            state: RoomState::Await as u8,
            time,
        }
    }

    ///检查准备状态
    pub fn check_ready(&self) -> bool {
        for team in self.teams.values() {
            let res = team.check_ready();
            if !res {
                return res;
            }
        }
        true
    }

    ///获取下一个行动单位
    pub fn get_last_action_mut(&mut self) -> Option<&mut ActionUnit> {
        let result = self.orders.last_mut();
        result
    }

    ///获取房号
    pub fn get_room_id(&self) -> u64 {
        self.id
    }

    ///判断成员是否存在
    fn is_exist_member(&self, team_id: &u8, user_id: &u32) -> bool {
        let mut result = self.teams.contains_key(team_id);
        if !result {
            return result;
        }
        let team = self.teams.get(team_id).unwrap();
        team.is_exist_member(user_id)
    }

    ///获得玩家的可变指针
    fn get_member_mut(&mut self, team_id: &u8, user_id: &u32) -> Option<&mut Member> {
        let mut result = self.teams.contains_key(team_id);
        if !result {
            return None;
        }
        let mut team = self.teams.get_mut(team_id).unwrap();
        team.get_member_mut(user_id)
    }

    ///获得玩家数量
    pub fn get_member_count(&self) -> usize {
        let mut size: usize = 0;
        for team in self.teams.values() {
            size += team.get_member_count();
        }
        size
    }

    ///添加成员
    pub fn add_member(&mut self, member: Member) {
        let mut size = self.teams.len() as u8;
        size += 1;
        let mut team = Team::default();
        let user_id = member.user_id;
        team.add_member(member);
        team.id = size;
        self.teams.insert(size + 1, team);
        self.player_team.insert(user_id, size);
    }

    ///移除玩家
    pub fn remove_member(&mut self, user_id: &u32) -> Option<Member> {
        let source_team_id = self.player_team.get_mut(user_id);
        if source_team_id.is_none() {
            return None;
        }
        let source_team = self.teams.get_mut(source_team_id.unwrap());
        if source_team.is_none() {
            return None;
        }
        let source_team = source_team.unwrap();
        self.player_team.remove(user_id);
        source_team.remove_member(user_id)
    }

    ///换队伍
    pub fn change_team(&mut self, user_id: &u32, team_id: &u8) {
        let member = self.remove_member(user_id);
        if member.is_none() {
            return;
        }
        let team = self.teams.get_mut(team_id);
        if team.is_none() {
            return;
        }
        let team = team.unwrap();
        team.add_member(member.unwrap());
        self.player_team.insert(*user_id, *team_id);
    }
}
