use super::*;
use crate::entity::member::Member;
use crate::entity::team::Team;
use chrono::{DateTime, Local, Utc};
use std::collections::HashMap;
use std::sync::atomic::Ordering;

///房间结构体，封装房间必要信息
pub struct Room {
    id: u32,
    map_id: u32,
    teams: HashMap<u32, Team>,
    time: DateTime<Utc>,
}

impl Room {
    fn new(map_id: u32) -> Room {
        let id: u32 = crate::ROOM_ID.fetch_add(10, Ordering::Relaxed);
        let teams: HashMap<u32, Team> = HashMap::new();
        let time = Utc::now();
        Room {
            id,
            map_id,
            teams,
            time,
        }
    }

    fn get_room_id(&self) -> u32 {
        self.id
    }

    fn is_exist_member(&self, team_id: &u32, user_id: &u32) -> bool {
        let mut result = self.teams.contains_key(team_id);
        if !result {
            return result;
        }
        let team = self.teams.get(team_id).unwrap();
        team.is_exist_member(user_id)
    }

    fn get_member_mut(&mut self, team_id: &u32, user_id: &u32) -> Option<&mut Member> {
        let mut result = self.teams.contains_key(team_id);
        if !result {
            return None;
        }
        let mut team = self.teams.get_mut(team_id).unwrap();
        team.get_member_mut(user_id)
    }
}
