use crate::entity::battle_model::RoomSetting;
use crate::entity::map_data::TileMap;
use crate::entity::member::{Member, MemberState};
use chrono::{DateTime, Utc};
use protobuf::Message;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use tools::cmd_code::ClientCode;
use tools::protos::base::{CharacterPt, MemberPt, RoomPt, TileMapPt};
use tools::protos::room::S_ROOM_MEMBER_NOTICE;
use tools::tcp::TcpSender;
use tools::templates::tile_map_temp::TileMapTemp;
use tools::util::packet::Packet;

enum RoomMemberNoticeType {
    AddMember = 1,
    UpdateMember = 2,
    LeaveMmeber = 3,
}

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
    id: u32,                           //房间id
    owner_id: u32,                     //房主id
    tile_map: TileMap,                 //地图数据
    pub members: HashMap<u32, Member>, //玩家对应的队伍
    orders: Vec<ActionUnit>,           //action队列
    state: u8,                         //房间状态
    setting: RoomSetting,              //房间设置
    room_type: u8,                     //房间类型
    pub sender: TcpSender,             //sender
    time: DateTime<Utc>,               //房间创建时间
}

impl Room {
    ///构建一个房间的结构体
    pub fn new(owner: Member, room_type: u8, sender: TcpSender) -> anyhow::Result<Room> {
        //转换成tilemap数据
        let tile_map = TileMap::default();
        let id: u32 = crate::ROOM_ID.fetch_add(10, Ordering::Relaxed);
        let time = Utc::now();
        let orders: Vec<ActionUnit> = Vec::new();
        let members: HashMap<u32, Member> = HashMap::new();
        let setting = RoomSetting::default();
        let mut room = Room {
            id,
            owner_id: owner.user_id,
            tile_map,
            members,
            orders,
            state: RoomState::Await as u8,
            setting,
            room_type,
            sender,
            time,
        };
        room.add_member(owner);
        Ok(room)
    }

    ///推送消息
    pub fn room_member_notice(&mut self, notice_type: u8, member: &Member) {
        let mut srmn = S_ROOM_MEMBER_NOTICE::new();
        srmn.set_notice_type(notice_type as u32);

        let mp = member_2_memberpt(member);
        srmn.set_member(mp);

        let mut packet = Packet::new(ClientCode::RoomMemberNotice as u32, 0, 0);
        packet.set_data_from_vec(srmn.write_to_bytes().unwrap());
        for (_, m) in self.members.iter() {
            if m.get_user_id() == member.user_id {
                continue;
            }
            packet.set_user_id(m.get_user_id());
            packet.set_is_broad(false);
            packet.set_is_client(true);
            self.sender.write(packet.build_server_bytes());
        }
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
        for (_, member) in self.members.iter() {
            let res = member.state == MemberState::Ready as u8;
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
        self.members.contains_key(user_id)
    }

    ///获得玩家的可变指针
    pub fn get_member_mut(&mut self, user_id: &u32) -> Option<&mut Member> {
        self.members.get_mut(user_id)
    }

    ///获得玩家的可变指针
    pub fn get_member_mut_by_user_id(&mut self, user_id: &u32) -> Option<&mut Member> {
        let result = self.members.get_mut(user_id);
        result
    }

    ///获得玩家数量
    pub fn get_member_count(&self) -> usize {
        self.members.len()
    }

    ///添加成员
    pub fn add_member(&mut self, mut member: Member) {
        let mut size = self.members.len() as u8;
        size += 1;
        member.team_id = size;
        let user_id = member.user_id;
        self.members.insert(user_id, member);
    }

    ///移除玩家
    pub fn remove_member(&mut self, user_id: &u32) -> Option<Member> {
        let res = self.members.remove(user_id);
        res
    }

    ///换队伍
    pub fn change_team(&mut self, user_id: &u32, team_id: &u8) {
        let member = self.get_member_mut(user_id);
        if member.is_none() {
            return;
        }
        let mut member = member.unwrap();
        member.team_id = *team_id;
    }

    ///T人
    pub fn kick_member(&mut self, user_id: &u32, target_id: &u32) -> Result<(), &str> {
        if self.owner_id != *user_id {
            return Err("不是房主，无法执行该操作");
        }
        if !self.members.contains_key(target_id) {
            return Err("该玩家不在房间内");
        }
        self.members.remove(target_id);
        Ok(())
    }

    ///判断房间是否有成员
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    ///转换成protobuf
    pub fn convert_to_pt(&self) -> RoomPt {
        let mut rp = RoomPt::new();
        rp.owner_id = self.owner_id;
        rp.room_id = self.get_room_id();
        for (_, member) in self.members.iter() {
            let mp = member_2_memberpt(member);
            rp.members.push(mp);
        }
        let mut tmp = TileMapPt::new();
        rp.set_tile_map(self.tile_map.convert_pt());
        rp
    }

    ///更换目标
    pub fn change_target(&mut self, user_id: &u32, target_id: &u32) -> anyhow::Result<()> {
        let target = self.members.contains_key(target_id);
        if !target {
            let s = format!(
                "this target player is not in this room!user_id:{},room_id:{}",
                target_id,
                self.get_room_id()
            );
            anyhow::bail!(s)
        }
        let member = self.members.get_mut(user_id);
        if member.is_none() {
            let s = format!(
                "this plauyer is not in this room!user_id:{},room_id:{}",
                user_id,
                self.get_room_id()
            );
            anyhow::bail!(s)
        }
        let member = member.unwrap();
        member.battle_cter.target_id = *target_id;
        Ok(())
    }
}

///Member转MemberPt
pub fn member_2_memberpt(member: &Member) -> MemberPt {
    let mut mp = MemberPt::new();
    mp.user_id = member.get_user_id();
    mp.state = member.state as u32;
    mp.nick_name = member.nick_name.clone();
    let mut cp = CharacterPt::new();
    cp.temp_id = member.battle_cter.temp_id;
    cp.set_skills(member.battle_cter.skills.clone());
    mp.set_cter(cp);
    mp
}
