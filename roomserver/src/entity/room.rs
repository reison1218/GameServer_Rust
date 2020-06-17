use crate::entity::battle_model::RoomSetting;
use crate::entity::map_data::TileMap;
use crate::entity::member::{Member, MemberState};
use chrono::{DateTime, Utc};
use log::error;
use protobuf::Message;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tools::cmd_code::ClientCode;
use tools::protos::base::{CharacterPt, MemberPt, RoomPt, RoomSettingPt, RoundTimePt};
use tools::protos::room::{
    S_CHANGE_TEAM, S_EMOJI, S_KICK_MEMBER, S_PREPARE_CANCEL, S_ROOM, S_ROOM_MEMBER_NOTICE,
    S_ROOM_NOTICE,
};
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

pub enum RoomMemberNoticeType {
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
    pub fn new(mut owner: Member, room_type: u8, sender: TcpSender) -> anyhow::Result<Room> {
        //转换成tilemap数据
        let tile_map = TileMap::default();
        let id: u32 = crate::ROOM_ID.fetch_add(1, Ordering::Relaxed);
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

        let mut size = room.members.len() as u8;
        size += 1;
        owner.team_id = size;
        room.members.insert(owner.user_id, owner);

        //返回客户端
        let mut sr = S_ROOM::new();
        sr.is_succ = true;
        sr.set_room(room.convert_to_pt());
        let bytes = Packet::build_packet_bytes(
            ClientCode::Room as u32,
            owner.user_id,
            sr.write_to_bytes().unwrap(),
            true,
            true,
        );
        let res = room.sender.write(bytes);
        if res.is_err() {
            let str = format!("{:?}", res.err().unwrap().to_string());
            error!("{:?}", str.as_str());
            anyhow::bail!("{:?}", str)
        }
        Ok(room)
    }

    pub fn prepare_cancel(&mut self, user_id: &u32, pregare_cancel: bool) {
        let member = self.members.get_mut(user_id);
        let mut spc = S_PREPARE_CANCEL::new();
        if member.is_none() {
            spc.is_succ = false;
            spc.err_mess = "this player not in the room!".to_owned();
        } else {
            let member = member.unwrap();
            if pregare_cancel {
                member.state = MemberState::Ready as u8;
            } else {
                member.state = MemberState::NotReady as u8;
            }
            //通知其他玩家
            self.room_member_notice(RoomMemberNoticeType::UpdateMember as u8, user_id);
        }

        //返回客户端
        spc.is_succ = true;
        spc.prepare = pregare_cancel;
        let bytes = Packet::build_packet_bytes(
            ClientCode::PrepareCancel as u32,
            *user_id,
            spc.write_to_bytes().unwrap(),
            true,
            true,
        );
        let res = self.sender.write(bytes);
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
        }
    }

    pub fn room_notice(&mut self, user_id: &u32) {
        let mut srn = S_ROOM_NOTICE::new();
        srn.owner_id = self.owner_id;
        let mut rs = RoomSettingPt::new();
        rs.battle_type = self.setting.battle_type as u32;
        rs.victory_condition = self.setting.victory_condition;
        rs.is_open_world_tile = self.setting.is_world_tile;
        let mut rt = RoundTimePt::new();
        rt.fixed_time = self.setting.round_time.fixed_time;
        rt.consume_time = self.setting.round_time.consume_time;
        rs.set_round_time(rt);
        srn.set_setting(rs);
        let mut packet = Packet::new(ClientCode::RoomNotice as u32, 0, 0);
        packet.set_data_from_vec(srn.write_to_bytes().unwrap());
        packet.set_is_client(true);
        packet.set_is_broad(false);
        for id in self.members.keys() {
            if *id == *user_id {
                continue;
            }
            packet.set_user_id(*id);
            let res = self.sender.write(packet.build_server_bytes());
            if res.is_err() {
                error!("{:?}", res.err().unwrap().to_string());
            }
        }
    }

    pub fn emoji(&mut self, user_id: u32, emoji_id: u32) {
        let mut packet = Packet::new(ClientCode::Emoji as u32, 0, 0);
        packet.set_is_client(true);
        let mut sej = S_EMOJI::new();
        sej.emoji_id = emoji_id;
        sej.user_id = user_id;
        packet.set_data_from_vec(sej.write_to_bytes().unwrap());
        for user_id in self.members.keys() {
            packet.set_user_id(*user_id);
            self.sender.write(packet.build_server_bytes());
        }
    }

    ///推送消息
    pub fn room_member_notice(&mut self, notice_type: u8, user_id: &u32) {
        let mut srmn = S_ROOM_MEMBER_NOTICE::new();
        srmn.set_notice_type(notice_type as u32);

        let member = self.members.get(user_id);
        if notice_type == RoomMemberNoticeType::LeaveMmeber as u8 {
            let mut mp = MemberPt::new();
            mp.user_id = *user_id;
            srmn.set_member(mp);
        } else {
            if member.is_none() {
                return;
            }
            let member = member.unwrap();
            let mp = member_2_memberpt(member);
            srmn.set_member(mp);
        }

        let mut packet = Packet::new(ClientCode::RoomMemberNotice as u32, 0, 0);
        packet.set_data_from_vec(srmn.write_to_bytes().unwrap());
        packet.set_is_broad(false);
        packet.set_is_client(true);
        for (_, m) in self.members.iter() {
            if m.get_user_id() == *user_id {
                continue;
            }
            packet.set_user_id(m.get_user_id());
            let res = self.sender.write(packet.build_server_bytes());
            if res.is_err() {
                error!("{:?}", res.err().unwrap().to_string());
            }
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
    pub fn get_member_ref(&self, user_id: &u32) -> Option<&Member> {
        self.members.get(user_id)
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

        //返回客户端消息
        let mut sr = S_ROOM::new();
        sr.is_succ = true;
        sr.set_room(self.convert_to_pt());
        let bytes = Packet::build_packet_bytes(
            ClientCode::Room as u32,
            user_id,
            sr.write_to_bytes().unwrap(),
            true,
            true,
        );
        let res = self.sender.write(bytes);
        if res.is_err() {
            let str = format!("{:?}", res.err().unwrap().to_string());
            error!("{:?}", str.as_str());
        }
        //通知房间里其他人
        self.room_member_notice(RoomMemberNoticeType::AddMember as u8, &user_id);
    }

    ///移除玩家
    pub fn remove_member(&mut self, user_id: &u32) -> Option<Member> {
        let res = self.members.remove(user_id);
        if res.is_some() {
            if self.get_owner_id() == *user_id && self.members.len() > 0 {
                for i in self.members.keys() {
                    self.owner_id = *i;
                    break;
                }
                self.room_notice(user_id);
            }
            self.room_member_notice(RoomMemberNoticeType::LeaveMmeber as u8, user_id);
        }
        res
    }

    ///换队伍
    pub fn change_team(&mut self, user_id: &u32, team_id: &u8) {
        let res = self.members.contains_key(user_id);
        if !res {
            return;
        }
        let mut sct = S_CHANGE_TEAM::new();
        sct.is_succ = true;
        let bytes = Packet::build_packet_bytes(
            ClientCode::Room as u32,
            *user_id,
            sct.write_to_bytes().unwrap(),
            true,
            true,
        );
        let res = self.sender.write(bytes);
        if res.is_err() {
            let str = format!("{:?}", res.err().unwrap().to_string());
            error!("{:?}", str.as_str());
        }
        let mut member = self.get_member_mut(user_id).unwrap();
        member.team_id = *team_id;
        //推送其他玩家
        self.room_member_notice(RoomMemberNoticeType::UpdateMember as u8, user_id);
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
        let mut skm = S_KICK_MEMBER::new();
        skm.is_succ = true;
        let bytes = Packet::build_packet_bytes(
            ClientCode::KickMember as u32,
            *user_id,
            skm.write_to_bytes().unwrap(),
            true,
            true,
        );
        let res = self.sender.write(bytes);
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
        }
        //通知其他成员
        self.room_member_notice(RoomMemberNoticeType::LeaveMmeber as u8, target_id);
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
    cp.temp_id = member.choiced_cter.temp_id;
    cp.set_skills(member.choiced_cter.skills.clone());
    cp.set_grade(member.choiced_cter.grade);
    mp.set_cter(cp);
    mp
}
