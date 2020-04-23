use super::*;
use chrono::{DateTime, Utc, Local};
use crate::entity::member::Member;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

///房间结构体，封装房间必要信息
pub struct Room{
    id:u32,
    map_id:u32,
    members:HashMap<u32,Member>,
    time:DateTime<Utc>,
}

impl Room {
    fn new(map_id:u32)->Room{
        let id:u32 = crate::ROOM_ID.fetch_add(10, Ordering::Relaxed);
        let members:HashMap<u32,Member> = HashMap::new();
        let time = Utc::now();
        Room{id,map_id,members,time}
    }

    fn get_room_id(&self)->u32{
        self.id
    }

    fn is_exist_member(&self,user_id:&u32)->bool{
        self.members.contains_key(user_id)
    }

    fn get_member_mut(&mut self,user_id:&u32)->Option<&mut Member>{
        self.members.get_mut(user_id)
    }
}