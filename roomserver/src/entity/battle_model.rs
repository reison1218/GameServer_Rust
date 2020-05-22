use super::*;
use crate::entity::member::{Member, MemberState, Target, UserType};
use crate::entity::room::Room;
use crate::template::template_contants::TILE_MAP_TEMPLATE;
use crate::template::templates::Template;
use crate::TEMPLATES;
use std::borrow::BorrowMut;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use tools::thread_pool::ThreadPoolType::user;

///战斗模式类型
#[derive(Debug, Copy, Clone)]
pub enum PVPModel {
    None = 0,            //初始值
    OneVOneVOneVOne = 1, //1v1v1v1
    TwoVTwo = 2,         //2v2
    OneVOne = 3,         //1v1
}

///房间缓存
#[derive(Debug, Copy, Clone, Default)]
pub struct RoomCache {
    room_id: u64,
    count: u32,
}

pub trait RoomModel {
    fn change_target(&mut self, user_id: &u32, target_id: &u32) -> Result<(), String> {
        let room = self.get_mut_room_by_user_id(user_id)?;
        room.change_target(user_id, target_id)?;
        Ok(())
    }
    fn check_is_in_room(&self, user_id: &u32) -> bool;
    fn create_room(&mut self, user_id: &u32, temp: &Template) -> Result<Room, String>;
    fn leave_room(&mut self, user_id: &u32) -> Result<(), String>;

    fn rm_room(&mut self, room_id: &u64) -> Result<(), String>;

    fn get_player_room_mut(&mut self) -> &mut HashMap<u32, u64>;

    fn get_rooms_mut(&mut self) -> &mut HashMap<u64, Room>;
    ///根据玩家id获得房间的可变指针
    fn get_mut_room_by_user_id(&mut self, user_id: &u32) -> Result<&mut Room, String> {
        let room_id = self.get_player_room_mut().get(user_id);
        if room_id.is_none() {
            let s = format!("this player is not in room,user_id:{}", user_id);
            error!("{}", s.as_str());
            return Err(s);
        }
        let room_id = *room_id.unwrap();
        let res = self.get_mut_room_by_room_id(&room_id)?;
        Ok(res)
    }

    ///根据房间id获得房间的可变指针
    fn get_mut_room_by_room_id(&mut self, room_id: &u64) -> Result<&mut Room, String> {
        let res = self.get_rooms_mut().get_mut(room_id);
        if res.is_none() {
            let s = format!("this room is not exit,room_id:{}", room_id);
            error!("{}", s.as_str());
            return Err(s);
        }
        Ok(res.unwrap())
    }
}

///好友房结构体
#[derive(Debug, Clone, Default)]
pub struct FriendRoom {
    pub player_room: HashMap<u32, u64>,
    pub rooms: HashMap<u64, Room>,
}

impl RoomModel for FriendRoom {
    ///校验是否在房间内
    fn check_is_in_room(&self, user_id: &u32) -> bool {
        self.player_room.contains_key(user_id)
    }

    ///创建房间
    fn create_room(&mut self, user_id: &u32, temp: &Template) -> Result<Room, String> {
        let room = Room::new(temp, user_id)?;
        self.player_room.insert(*user_id, room.get_room_id());
        self.rooms.insert(room.get_room_id(), room.clone());
        Ok(room)
    }

    ///离开房间
    fn leave_room(&mut self, user_id: &u32) -> Result<(), String> {
        let mut room = self.get_mut_room_by_user_id(user_id)?;
        room.remove_member(user_id);
        let room_id = room.get_room_id();
        //如果房间空了，则直接移除房间
        if room.is_empty() {
            self.rooms.remove(&room_id);
        }
        Ok(())
    }

    fn rm_room(&mut self, room_id: &u64) -> Result<(), String> {
        self.rooms.remove(room_id);
        Ok(())
    }

    fn get_player_room_mut(&mut self) -> &mut HashMap<u32, u64, RandomState> {
        self.player_room.borrow_mut()
    }

    fn get_rooms_mut(&mut self) -> &mut HashMap<u64, Room, RandomState> {
        self.rooms.borrow_mut()
    }
}

impl FriendRoom {
    ///T人
    pub fn kick_member(&mut self, user_id: &u32, target_id: &u32) -> Result<(), String> {
        let mut room = self.get_mut_room_by_user_id(user_id)?;
        if !room.is_exist_member(target_id) {
            let s = format!("this player is not in the room,target_id:{}", target_id);
            error!("{}", s.as_str());
            return Err(s);
        }
        if room.get_owner_id() != *user_id {
            let s = format!(
                "this player is not owner of room,user_id:{},room_id:{}",
                user_id,
                room.get_room_id()
            );
            error!("{}", s.as_str());
            return Err(s);
        }
        room.remove_member(target_id);
        self.player_room.remove(target_id);
        Ok(())
    }

    ///换队伍
    pub fn change_team(&mut self, user_id: &u32, team_id: &u8) -> Result<(), String> {
        let mut room = self.get_mut_room_by_user_id(user_id)?;
        room.change_team(user_id, team_id);
        Ok(())
    }
}

///等待匹配的玩家结构体
#[derive(Debug, Default, Clone)]
pub struct MatchPlayer {
    pub user_id: u32,
    pub model: u8,
}

///公共房结构体
#[derive(Debug, Default, Clone)]
pub struct PubRoom {
    pub model_type: u8,                 //战斗模式类型
    pub player_room: HashMap<u32, u64>, //key:玩家id    value:房间id
    pub rooms: HashMap<u64, Room>,      //key:房间id    value:房间结构体
    pub room_cache: Vec<RoomCache>,     //key:房间id    value:房间人数
}

impl RoomModel for PubRoom {
    ///校验是否在房间内
    fn check_is_in_room(&self, user_id: &u32) -> bool {
        self.player_room.contains_key(user_id)
    }

    ///创建房间
    fn create_room(&mut self, user_id: &u32, temp: &Template) -> Result<Room, String> {
        let room = Room::new(temp, user_id)?;
        self.player_room.insert(*user_id, room.get_room_id());
        self.rooms.insert(room.get_room_id(), room.clone());
        let mut rc = RoomCache::default();
        rc.room_id = room.get_room_id();
        rc.count = 4;
        self.room_cache.push(rc);
        Ok(room)
    }

    ///离开房间
    fn leave_room(&mut self, user_id: &u32) -> Result<(), String> {
        let mut room = self.get_mut_room_by_user_id(user_id)?;
        room.remove_member(user_id);
        if room.is_empty() {
            let room_id = room.get_room_id();
            self.rm_room(&room_id);
        }
        Ok(())
    }

    ///删除房间
    fn rm_room(&mut self, room_id: &u64) -> Result<(), String> {
        self.rooms.remove(room_id);
        self.remove_room_cache(room_id);
        Ok(())
    }

    fn get_player_room_mut(&mut self) -> &mut HashMap<u32, u64, RandomState> {
        self.player_room.borrow_mut()
    }

    fn get_rooms_mut(&mut self) -> &mut HashMap<u64, Room, RandomState> {
        self.rooms.borrow_mut()
    }
}

impl PubRoom {
    pub fn get_room_cache_mut(&mut self, room_id: &u64) -> Option<&mut RoomCache> {
        let res = self.room_cache.iter_mut().find(|x| x.room_id == *room_id);
        res
    }

    ///删除缓存房间
    pub fn remove_room_cache(&mut self, room_id: &u64) {
        let mut index: isize = -1;
        for i in self.room_cache.iter() {
            index += 1;
            if i.room_id != *room_id {
                continue;
            }
            break;
        }
        if index < 0 {
            return;
        }
        self.room_cache.remove(index as usize);
    }

    ///快速加入
    pub fn quickly_start(&mut self, user_id: &u32) -> Result<(), String> {
        let res = self.check_is_in_room(user_id);
        if res {
            let s = format!("this player already in the room!user_id:{}", user_id);
            error!("{}", s.as_str());
            return Err(s);
        }
        let map_id = 1002 as u64;
        //如果房间缓存里没有，则创建新房间
        if self.room_cache.is_empty() {
            //校验地图配置
            let res: Option<&Template> = TEMPLATES.get(TILE_MAP_TEMPLATE, &map_id);
            if res.is_none() {
                let s = format!("this map config is None,map_id:{}", map_id);
                error!("{}", s.as_str());
                return Err(s);
            }
            //创建房间
            let map_temp = res.unwrap();
            self.create_room(user_id, map_temp);
        } else {
            //如果有，则往房间里塞
            let mut room_cacahe = self.room_cache.last_mut().unwrap();
            let room_id = room_cacahe.room_id;
            let mut room = self.get_mut_room_by_room_id(&room_id)?;
            let mut member = Member {
                user_id: *user_id,
                nick_name: "test".to_string(),
                user_type: UserType::Real as u8,
                state: MemberState::NotReady as u8,
                target: Target::default(),
            };
            room.add_member(member);
            let mut room_cacahe = self.room_cache.last_mut().unwrap();
            room_cacahe.count += 1;
            //如果人满里，则从缓存房间列表中弹出
            if room_cacahe.count >= 4 {
                self.room_cache.pop();
            }
            self.room_cache
                .sort_by(|a, b| a.count.partial_cmp(&b.count).unwrap());
        }
        Ok(())
    }
}
