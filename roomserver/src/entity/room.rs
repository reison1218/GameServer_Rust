use crate::entity::battle_model::RoomSetting;
use crate::entity::map_data::TileMap;
use crate::entity::member::{Member, MemberState, Target};
use crate::entity::team::Team;
use chrono::{DateTime, Utc};
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tools::protos::base::RoomPt;
use tools::templates::tile_map_temp::TileMapTemp;

pub enum RoomState {
    Await = 0,   //等待
    Started = 1, //已经开始
}

pub enum Permission {
    Private = 0, //私有房间
    Public = 1,  //公开房间
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
    id: u32,                       //房间id
    owner_id: u32,                 //房主id
    tile_map: TileMap,             //地图数据
    player_team: HashMap<u32, u8>, //玩家对应的队伍
    teams: HashMap<u8, Team>,      //队伍数据
    orders: Vec<ActionUnit>,       //action队列
    state: u8,                     //房间状态
    setting: RoomSetting,          //房间设置
    room_type: u8,                 //房间类型
    time: DateTime<Utc>,           //房间创建时间
}

impl Room {
    ///构建一个房间的结构体
    pub fn new(owner: Member, room_type: u8) -> anyhow::Result<Room> {
        //转换成tilemap数据
        let tile_map = TileMap::default();
        let id: u32 = crate::ROOM_ID.fetch_add(10, Ordering::Relaxed);
        let time = Utc::now();
        let teams: HashMap<u8, Team> = HashMap::new();
        let orders: Vec<ActionUnit> = Vec::new();
        let player_team: HashMap<u32, u8> = HashMap::new();
        let setting = RoomSetting::default();
        let mut room = Room {
            id,
            owner_id: owner.user_id,
            tile_map,
            player_team,
            teams,
            orders,
            state: RoomState::Await as u8,
            setting,
            room_type,
            time,
        };
        room.add_member(owner);
        Ok(room)
    }

    pub fn get_status(&self) -> u8 {
        self.state
    }

    pub fn set_status(&mut self, status: u8) -> u8 {
        self.state = status;
        self.state
    }

    pub fn set_room_setting(&mut self, setting: RoomSetting) {
        self.setting = setting;
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

    ///获得房主ID
    pub fn get_owner_id(&self) -> u32 {
        self.owner_id
    }

    ///获得房间类型
    pub fn get_room_type(&self) -> u8 {
        self.room_type
    }

    ///获取房号
    pub fn get_room_id(&self) -> u32 {
        self.id
    }

    ///判断成员是否存在
    pub fn is_exist_member(&self, user_id: &u32) -> bool {
        self.player_team.contains_key(user_id)
    }

    ///获得玩家的可变指针
    pub fn get_member_mut(&mut self, team_id: &u8, user_id: &u32) -> Option<&mut Member> {
        let result = self.teams.contains_key(team_id);
        if !result {
            return None;
        }
        let team = self.teams.get_mut(team_id).unwrap();
        team.get_member_mut(user_id)
    }

    ///获得玩家的可变指针
    pub fn get_member_mut_by_user_id(&mut self, user_id: &u32) -> Option<&mut Member> {
        let result = self.player_team.get(user_id);
        if result.is_none() {
            return None;
        }
        let team_id = result.unwrap();
        let team = self.teams.get_mut(team_id);
        if team.is_none() {
            return None;
        }
        let team = team.unwrap();
        team.members.get_mut(user_id)
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
        self.teams.insert(size, team);
        self.player_team.insert(user_id, size);
    }

    ///移除玩家
    pub fn remove_member(&mut self, user_id: &u32) -> Option<Member> {
        let team_id = self.player_team.get_mut(user_id);
        if team_id.is_none() {
            return None;
        }
        let team = self.teams.get_mut(team_id.unwrap());
        if team.is_none() {
            return None;
        }
        let team = team.unwrap();
        let res = self.player_team.remove(user_id);
        team.remove_member(user_id)
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

    ///T人
    pub fn kick_member(&mut self, user_id: &u32, target_id: &u32) -> Result<(), &str> {
        if self.owner_id != *user_id {
            return Err("不是房主，无法执行该操作");
        }
        if !self.player_team.contains_key(target_id) {
            return Err("该玩家不在房间内");
        }
        let team_id = self.player_team.get(target_id).unwrap();
        let team = self.teams.get_mut(team_id);
        if team.is_none() {
            return Err("队伍不存在");
        }
        let team = team.unwrap();
        team.members.remove(target_id);
        //如果队伍没人了，直接删除队伍
        if team.members.len() == 0 {
            self.teams.remove(team_id);
        }
        self.player_team.remove(target_id);
        Ok(())
    }

    pub fn get_teams(&self) -> Iter<u8, Team> {
        let res = self.teams.iter();
        res
    }

    ///判断房间是否有成员
    pub fn is_empty(&self) -> bool {
        for i in self.teams.iter() {
            if !i.1.members.is_empty() {
                return false;
            }
        }
        true
    }

    ///转换成protobuf
    pub fn convert_to_pt(&self) -> RoomPt {
        let mut v = Vec::new();
        for (_, team) in self.teams.iter() {
            let team_pt = team.convert_to_pt();
            v.push(team_pt);
        }
        let mut rp = RoomPt::new();
        rp.owner_id = self.owner_id;
        rp.room_id = self.get_room_id();
        let res = protobuf::RepeatedField::from(v);
        rp.set_teams(res);
        rp.set_tile_map(self.tile_map.convert_pt());
        rp
    }

    ///更换目标
    pub fn change_target(&mut self, user_id: &u32, target_id: &u32) -> anyhow::Result<()> {
        let team_id = self.player_team.get(user_id);
        if team_id.is_none() {
            let s = format!(
                "this player is not in this room!,user_id:{},room_id:{}",
                user_id,
                self.get_room_id()
            );
            anyhow::bail!(s)
        }
        let team_id = *team_id.unwrap();
        let team_id = &team_id;
        let target_team_id = self.player_team.get(target_id);
        if target_team_id.is_none() {
            let s = format!(
                "this target_player is not in this room!user_id:{},room_id:{}",
                target_id,
                self.get_room_id()
            );
            anyhow::bail!(s)
        }
        let target_team_id = *target_team_id.unwrap();
        let target_team_id = &target_team_id;

        let member = self.get_member_mut(team_id, user_id).unwrap();
        let mut target = Target::default();
        target.team_id = *target_team_id;
        target.user_id = *target_id;
        member.battle_cters.target = target;
        Ok(())
    }
}
