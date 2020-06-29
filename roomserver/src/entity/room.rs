use crate::entity::map_data::TileMap;
use crate::entity::member::{Member, MemberState};
use crate::entity::room_model::RoomSetting;
use chrono::{DateTime, Local, Utc};
use log::error;
use protobuf::Message;
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::str::FromStr;
use tools::cmd_code::ClientCode;
use tools::protos::base::{MemberPt, RoomPt, RoomSettingPt, RoundTimePt};
use tools::protos::room::{
    S_CHANGE_TEAM, S_EMOJI, S_EMOJI_NOTICE, S_KICK_MEMBER, S_PREPARE_CANCEL, S_ROOM,
    S_ROOM_MEMBER_NOTICE, S_ROOM_NOTICE,
};
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

//最大成员数量
pub const MEMBER_MAX: u8 = 4;

pub enum RoomMemberNoticeType {
    AddMember = 1,
    UpdateMember = 2,
    LeaveMmeber = 3,
}

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
    id: u32,                           //房间id
    owner_id: u32,                     //房主id
    tile_map: TileMap,                 //地图数据
    pub members: HashMap<u32, Member>, //玩家对应的队伍
    pub member_index: [u32; 4],        //玩家对应的位置
    orders: Vec<ActionUnit>,           //action队列
    state: u8,                         //房间状态
    pub setting: RoomSetting,          //房间设置
    room_type: u8,                     //房间类型
    pub sender: TcpSender,             //sender
    time: DateTime<Utc>,               //房间创建时间
}

impl Room {
    ///构建一个房间的结构体
    pub fn new(mut owner: Member, room_type: u8, sender: TcpSender) -> anyhow::Result<Room> {
        //转换成tilemap数据
        let user_id = owner.user_id;
        let tile_map = TileMap::default();
        let mut str = Local::now().timestamp_subsec_micros().to_string();
        str.push_str(thread_rng().gen_range(1, 999).to_string().as_str());
        let id: u32 = u32::from_str(str.as_str())?;
        let time = Utc::now();
        let orders: Vec<ActionUnit> = Vec::new();
        let members: HashMap<u32, Member> = HashMap::new();
        let setting = RoomSetting::default();
        let member_index = [0; MEMBER_MAX as usize];
        let mut room = Room {
            id,
            owner_id: owner.user_id,
            tile_map,
            members,
            member_index,
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
        owner.join_time = Local::now().timestamp_millis() as u64;
        room.members.insert(user_id, owner);
        room.member_index[0] = user_id;
        //返回客户端
        let mut sr = S_ROOM::new();
        sr.is_succ = true;
        sr.set_room(room.convert_to_pt());
        let bytes = Packet::build_packet_bytes(
            ClientCode::Room as u32,
            user_id,
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

    ///检查角色
    pub fn check_character(&self, cter_id: u32) -> anyhow::Result<()> {
        for cter in self.members.values() {
            if cter_id > 0 && cter.chose_cter.temp_id == cter_id {
                let str = format!("this character was choiced!cter_id:{}", cter_id);
                anyhow::bail!(str)
            }
        }
        Ok(())
    }

    ///准备
    pub fn prepare_cancel(&mut self, user_id: &u32, pregare_cancel: bool) {
        let member = self.members.get_mut(user_id).unwrap();
        match pregare_cancel {
            true => member.state = MemberState::Ready as u8,
            false => member.state = MemberState::NotReady as u8,
        }
        //通知其他玩家
        let mut spc = S_PREPARE_CANCEL::new();
        spc.is_succ = true;
        self.room_member_notice(RoomMemberNoticeType::UpdateMember as u8, user_id);
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

    ///房间变更通知
    pub fn room_notice(&mut self, user_id: &u32) {
        let mut srn = S_ROOM_NOTICE::new();
        srn.owner_id = self.owner_id;
        srn.set_setting(self.setting.clone().into());
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

    ///发送表情包
    pub fn emoji(&mut self, user_id: u32, emoji_id: u32) {
        let mut packet = Packet::new(ClientCode::Emoji as u32, 0, 0);
        packet.set_is_client(true);
        packet.set_user_id(user_id);
        //回给发送人
        let mut sej = S_EMOJI::new();
        sej.is_succ = true;
        packet.set_data_from_vec(sej.write_to_bytes().unwrap());
        let res = self.sender.write(packet.build_server_bytes());
        if res.is_err() {
            error!("{:?}", res.err().unwrap());
        }
        //推送给房间其他人

        let mut sen = S_EMOJI_NOTICE::new();
        sen.user_id = user_id;
        sen.emoji_id = emoji_id;
        for user_id in self.members.keys() {
            if *user_id == packet.get_user_id() {
                continue;
            }

            let bytes = Packet::build_packet_bytes(
                ClientCode::EmojiNotice as u32,
                *user_id,
                sen.write_to_bytes().unwrap(),
                true,
                true,
            );
            let res = self.sender.write(bytes);
            match res {
                Err(e) => error!("{:?}", e.to_string()),
                Ok(_) => {}
            }
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
            let mp = member.unwrap().clone().into();
            srmn.set_member(mp);
        }

        let mut packet = Packet::new(ClientCode::RoomMemberNotice as u32, 0, 0);
        packet.set_data_from_vec(srmn.write_to_bytes().unwrap());
        packet.set_is_broad(false);
        packet.set_is_client(true);
        for (_, m) in self.members.iter() {
            if m.get_user_id() == *user_id && RoomMemberNoticeType::LeaveMmeber as u8 != notice_type
            {
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
        for member in self.members.values() {
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
    pub fn add_member(&mut self, mut member: Member) -> anyhow::Result<u32> {
        let mut size = self.members.len() as u8;
        let user_id = member.user_id;
        size += 1;
        member.team_id = size;
        member.join_time = Local::now().timestamp_millis() as u64;
        self.members.insert(user_id, member);
        for i in 0..self.member_index.len() - 1 {
            if self.member_index[i] != 0 {
                continue;
            }
            self.member_index[i] = user_id;
            break;
        }

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
        Ok(self.id)
    }

    ///移除玩家
    pub fn remove_member(&mut self, user_id: &u32) -> Option<Member> {
        let res = self.members.remove(user_id);
        if res.is_some() {
            for i in 0..self.member_index.len() - 1 {
                if self.member_index[i] != *user_id {
                    continue;
                }
                self.member_index[i] = 0;
                break;
            }

            if self.get_owner_id() == *user_id && self.get_member_count() > 0 {
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
        //通知其他成员
        self.room_member_notice(RoomMemberNoticeType::LeaveMmeber as u8, target_id);
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
        rp.set_room_type(self.get_room_type() as u32);
        for user_id in self.member_index.iter() {
            let member = self.members.get(user_id);
            if member.is_some() {
                let member = member.unwrap();
                let mp = member.clone().into();
                rp.members.push(mp);
            } else {
                let mp = MemberPt::new();
                rp.members.push(mp);
            }
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
                "this player is not in this room!user_id:{},room_id:{}",
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
