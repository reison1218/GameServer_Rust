use super::*;
use crate::entity::member::{Member, MemberState, Target, UserType};
use crate::entity::room::Room;
use crate::TEMPLATES;
use serde::export::Result::Err;
use std::borrow::BorrowMut;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::error::Error;
use tools::templates::template::TemplateMgrTrait;
use tools::templates::tile_map_temp::{TileMapTemp, TileMapTempMgr};

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
    room_id: u32,
    count: u32,
}

pub trait RoomModel {
    fn change_target(&mut self, user_id: &u32, target_id: &u32) -> Result<()> {
        let room = self.get_mut_room_by_user_id(user_id)?;
        room.change_target(user_id, target_id)?;
        Ok(())
    }
    fn check_is_in_room(&self, user_id: &u32) -> bool;
    fn create_room(&mut self, user_id: &u32, temp: &TileMapTemp) -> Result<Room>;
    fn leave_room(&mut self, user_id: &u32) -> Result<()>;

    fn rm_room(&mut self, room_id: &u32) -> Result<()>;

    fn get_player_room_mut(&mut self) -> &mut HashMap<u32, u32>;

    fn get_rooms_mut(&mut self) -> &mut HashMap<u32, Room>;
    ///根据玩家id获得房间的可变指针
    fn get_mut_room_by_user_id(&mut self, user_id: &u32) -> Result<&mut Room> {
        let room_id = self.get_player_room_mut().get(user_id);
        if room_id.is_none() {
            let s = format!("this player is not in room,user_id:{}", user_id);
            return bail!(s);
        }
        let room_id = *room_id.unwrap();
        let res = self.get_mut_room_by_room_id(&room_id)?;
        Ok(res)
    }

    ///根据房间id获得房间的可变指针
    fn get_mut_room_by_room_id(&mut self, room_id: &u32) -> Result<&mut Room> {
        let res = self.get_rooms_mut().get_mut(room_id);
        if res.is_none() {
            let s = format!("this room is not exit,room_id:{}", room_id);
            return bail!(s);
        }
        Ok(res.unwrap())
    }
}

///好友房结构体
#[derive(Debug, Clone, Default)]
pub struct FriendRoom {
    pub player_room: HashMap<u32, u32>,
    pub rooms: HashMap<u32, Room>,
}

impl RoomModel for FriendRoom {
    ///校验是否在房间内
    fn check_is_in_room(&self, user_id: &u32) -> bool {
        self.player_room.contains_key(user_id)
    }

    ///创建房间
    fn create_room(&mut self, user_id: &u32, temp: &TileMapTemp) -> Result<Room> {
        let room = Room::new(temp, user_id)?;
        self.player_room.insert(*user_id, room.get_room_id());
        self.rooms.insert(room.get_room_id(), room.clone());
        Ok(room)
    }

    ///离开房间
    fn leave_room(&mut self, user_id: &u32) -> tools::result::errors::Result<()> {
        let mut room = self.get_mut_room_by_user_id(user_id)?;
        room.remove_member(user_id);
        let room_id = room.get_room_id();
        //如果房间空了，则直接移除房间
        if room.is_empty() {
            self.rooms.remove(&room_id);
        }
        Ok(())
    }

    fn rm_room(&mut self, room_id: &u32) -> Result<()> {
        self.rooms.remove(room_id);
        Ok(())
    }

    fn get_player_room_mut(&mut self) -> &mut HashMap<u32, u32, RandomState> {
        self.player_room.borrow_mut()
    }

    fn get_rooms_mut(&mut self) -> &mut HashMap<u32, Room, RandomState> {
        self.rooms.borrow_mut()
    }
}

impl FriendRoom {
    ///T人
    pub fn kick_member(&mut self, user_id: &u32, target_id: &u32) -> Result<()> {
        let mut room = self.get_mut_room_by_user_id(user_id)?;
        if !room.is_exist_member(target_id) {
            let s = format!("this player is not in the room,target_id:{}", target_id);
            return bail!(s);
        }
        if room.get_owner_id() != *user_id {
            let s = format!(
                "this player is not owner of room,user_id:{},room_id:{}",
                user_id,
                room.get_room_id()
            );
            return bail!(s);
        }
        room.remove_member(target_id);
        self.player_room.remove(target_id);
        Ok(())
    }

    ///换队伍
    pub fn change_team(&mut self, user_id: &u32, team_id: &u8) -> Result<()> {
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
    pub player_room: HashMap<u32, u32>, //key:玩家id    value:房间id
    pub rooms: HashMap<u32, Room>,      //key:房间id    value:房间结构体
    pub room_cache: Vec<RoomCache>,     //key:房间id    value:房间人数
}

impl RoomModel for PubRoom {
    ///校验是否在房间内
    fn check_is_in_room(&self, user_id: &u32) -> bool {
        self.player_room.contains_key(user_id)
    }

    ///创建房间
    fn create_room(&mut self, user_id: &u32, temp: &TileMapTemp) -> Result<Room> {
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
    fn leave_room(&mut self, user_id: &u32) -> Result<()> {
        let mut room = self.get_mut_room_by_user_id(user_id)?;
        room.remove_member(user_id);
        if room.is_empty() {
            let room_id = room.get_room_id();
            self.rm_room(&room_id);
        }
        Ok(())
    }

    ///删除房间
    fn rm_room(&mut self, room_id: &u32) -> Result<()> {
        self.rooms.remove(room_id);
        self.remove_room_cache(room_id);
        Ok(())
    }

    fn get_player_room_mut(&mut self) -> &mut HashMap<u32, u32, RandomState> {
        self.player_room.borrow_mut()
    }

    fn get_rooms_mut(&mut self) -> &mut HashMap<u32, Room, RandomState> {
        self.rooms.borrow_mut()
    }
}

impl PubRoom {
    pub fn get_room_cache_mut(&mut self, room_id: &u32) -> Option<&mut RoomCache> {
        let res = self.room_cache.iter_mut().find(|x| x.room_id == *room_id);
        res
    }

    ///删除缓存房间
    pub fn remove_room_cache(&mut self, room_id: &u32) {
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
    pub fn quickly_start(&mut self, user_id: &u32) -> Result<()> {
        let res = self.check_is_in_room(user_id);
        if res {
            let s = format!("this player already in the room!user_id:{}", user_id);
            return bail!(s);
        }
        let map_id = 1002 as u32;
        //如果房间缓存里没有，则创建新房间
        if self.room_cache.is_empty() {
            //校验地图配置
            let room_tmp_ref: &TileMapTempMgr = TEMPLATES.get_tile_map_ref();
            if room_tmp_ref.is_empty() {
                let s = format!("this map config is None,map_id:{}", map_id);
                return bail!(s);
            }
            //创建房间
            let res = room_tmp_ref.get_temp(map_id)?;
            self.create_room(user_id, res);
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
