use crate::room::member::{Member, MemberState, PunishMatch};
use crate::room::room_model::{RoomSetting, RoomType};
use crate::task_timer::Task;
use crate::TEMPLATES;
use chrono::{DateTime, Local, Utc};
use crossbeam::channel::Sender;
use log::{error, warn};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use protobuf::Message;
use rand::{thread_rng, Rng};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::str::FromStr;
use tools::cmd_code::{BattleCode, ClientCode, GameCode};
use tools::macros::GetMutRef;
use tools::protos::base::{MemberPt, RoomPt};
use tools::protos::room::{
    S_CHANGE_TEAM_NOTICE, S_EMOJI, S_EMOJI_NOTICE, S_KICK_MEMBER, S_MATCH_SUCCESS_NOTICE,
    S_PREPARE_CANCEL, S_PREPARE_CANCEL_NOTICE, S_ROOM, S_ROOM_ADD_MEMBER_NOTICE,
    S_ROOM_MEMBER_LEAVE_NOTICE, S_ROOM_NOTICE,
};
use tools::protos::server_protocol::{B_R_G_PUNISH_MATCH, R_B_START};
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

///最大成员数量
pub const MEMBER_MAX: u8 = 4;

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RoomSettingType {
    None = 0,
    IsOpenAI = 1,
    SeasonId = 2,
    TurnLimitTime = 3,
}

impl From<u32> for RoomSettingType {
    fn from(value: u32) -> Self {
        match value {
            1 => RoomSettingType::IsOpenAI,
            2 => RoomSettingType::SeasonId,
            3 => RoomSettingType::TurnLimitTime,
            _ => RoomSettingType::None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RoomMemberNoticeType {
    None = 0,         //无效
    UpdateMember = 2, //更新成员
}

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum MemberLeaveNoticeType {
    None = 0,   //无效
    Leave = 1,  //自己离开
    Kicked = 2, //被T
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RoomState {
    AwaitConfirm = 0, //等待进入 只有匹配模式才会有到壮体啊
    Await = 1,        //等待
    ChoiceIndex = 2,  //选择占位
}

///房间结构体，封装房间必要信息
#[derive(Clone)]
pub struct Room {
    id: u32,                                      //房间id
    room_type: RoomType,                          //房间类型
    owner_id: u32,                                //房主id
    pub state: RoomState,                         //房间状态
    pub members: HashMap<u32, Member>,            //玩家对应的队伍
    pub member_index: [u32; MEMBER_MAX as usize], //玩家对应的位置
    pub setting: RoomSetting,                     //房间设置
    pub tcp_sender: TcpSender,                    //tcpsender
    task_sender: Sender<Task>,                    //任务sender
    time: DateTime<Utc>,                          //房间创建时间
}

tools::get_mut_ref!(Room);

impl From<&Room> for RoomPt {
    fn from(room: &Room) -> Self {
        let mut rp = RoomPt::new();
        rp.room_id = room.get_room_id();
        rp.room_type = room.get_room_type().into_u32();
        rp.room_status = room.state as u32;
        let setting = room.setting.borrow();
        rp.set_setting(setting.into());
        rp.owner_id = room.owner_id;
        for member in room.members.values() {
            let mp = member.clone().into();
            rp.members.push(mp);
        }
        rp
    }
}

impl Room {
    ///构建一个房间的结构体
    pub fn new(
        mut owner: Member,
        room_type: RoomType,
        sender: TcpSender,
        task_sender: Sender<Task>,
    ) -> anyhow::Result<Room> {
        //转换成tilemap数据
        let user_id = owner.user_id;
        let mut str = Local::now().timestamp_subsec_micros().to_string();
        str.push_str(thread_rng().gen_range(1..999).to_string().as_str());
        let id: u32 = u32::from_str(str.as_str())?;
        let time = Utc::now();
        let mut room_state = RoomState::Await;
        if room_type == RoomType::OneVOneVOneVOneMatch {
            room_state = RoomState::AwaitConfirm;
            owner.state = MemberState::AwaitConfirm;
        }
        let mut room = Room {
            id,
            owner_id: user_id,
            members: HashMap::new(),
            member_index: [0; MEMBER_MAX as usize],
            state: room_state,
            setting: RoomSetting::default(),
            room_type,
            tcp_sender: sender,
            task_sender,
            time,
        };
        if room.room_type == RoomType::OneVOneVOneVOneMatch {
            let limit_time = TEMPLATES
                .constant_temp_mgr()
                .temps
                .get("battle_turn_limit_time");
            if let Some(limit_time) = limit_time {
                let res = u32::from_str(limit_time.value.as_str());
                if let Err(e) = res {
                    error!("{:?}", e);
                } else {
                    room.setting.turn_limit_time = res.unwrap();
                }
            } else {
                warn!("constant temp's battle_turn_limit_time is none!")
            }
        }
        let mut size = room.members.len() as u8;
        size += 1;
        owner.team_id = size;
        owner.join_time = Local::now().timestamp_millis() as u64;
        room.members.insert(user_id, owner);
        room.member_index[0] = user_id;
        if room.room_type != RoomType::OneVOneVOneVOneMatch {
            //返回客户端
            let mut sr = S_ROOM::new();
            sr.is_succ = true;
            sr.set_room(room.convert_to_pt());
            room.send_2_client(ClientCode::Room, user_id, sr.write_to_bytes().unwrap());
        }
        Ok(room)
    }

    ///离开房间检查是否需要添加惩罚
    pub fn check_punish_for_leave(&mut self, user_id: u32) {
        if self.room_type != RoomType::OneVOneVOneVOneMatch {
            return;
        }
        let member_count = self.members.len();
        //判断是否需要重制
        let member = self.members.get_mut(&user_id);
        if let None = member {
            return;
        }
        let member = member.unwrap();
        let mut res: Option<PunishMatch> = None;
        //判断人满了没，满了就惩罚
        if member_count == MEMBER_MAX as usize {
            member.punish_match.add_punish();
            res = Some(member.punish_match);
        }
        //同步到游戏服
        if res.is_none() {
            return;
        }
        let pm = res.unwrap();
        let mut brg = B_R_G_PUNISH_MATCH::new();
        brg.set_punish_match(pm.into());
        let bytes = brg.write_to_bytes();
        match bytes {
            Ok(bytes) => {
                self.send_2_server(GameCode::SyncPunish.into_u32(), user_id, bytes);
            }
            Err(e) => {
                warn!("{:?}", e);
            }
        }
    }

    pub fn check_all_confirmed_into_room(&self) -> bool {
        for member in self.members.values() {
            if member.state == MemberState::AwaitConfirm {
                return false;
            }
        }
        true
    }

    ///转发到游戏中心服
    pub fn send_2_server(&mut self, cmd: u32, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd, user_id, bytes, true, false);
        self.tcp_sender.send(bytes);
    }

    pub fn get_member_vec(&self) -> Vec<u32> {
        let mut v = Vec::new();
        for member in self.members.keys() {
            v.push(*member);
        }
        v
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let member = self.members.get(&user_id);
        if let None = member {
            return;
        }
        let member = member.unwrap();
        //如果是机器人，则返回，不发送
        if member.is_robot {
            return;
        }
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, true);
        self.tcp_sender.send(bytes);
    }

    pub fn push_match_success(&mut self) {
        let mut user_id;
        let smsn = S_MATCH_SUCCESS_NOTICE::new();
        let bytes = smsn.write_to_bytes().unwrap();
        for member in self.members.values() {
            user_id = member.user_id;
            //如果是机器人，则返回，不发送
            if member.is_robot {
                continue;
            }
            if member.state != MemberState::AwaitConfirm {
                continue;
            }
            let res = Packet::build_packet_bytes(
                ClientCode::MatchSuccessNotice.into_u32(),
                user_id,
                bytes.clone(),
                true,
                true,
            );
            self.tcp_sender.send(res);
        }
    }

    pub fn send_2_all_client(&mut self, cmd: ClientCode, bytes: Vec<u8>) {
        let mut user_id;
        for member in self.members.values() {
            user_id = member.user_id;
            //如果是机器人，则返回，不发送
            if member.is_robot {
                continue;
            }
            let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes.clone(), true, true);
            self.tcp_sender.send(bytes);
        }
    }

    ///检查角色
    pub fn check_character(&self, cter_id: u32) -> anyhow::Result<()> {
        for cter in self.members.values() {
            if cter_id > 0 && cter.chose_cter.cter_id == cter_id {
                let str = format!("this character was choiced!cter_id:{}", cter_id);
                anyhow::bail!(str)
            }
        }
        Ok(())
    }

    pub fn do_cancel_prepare(&mut self) {
        let members_ptr = self.members.borrow_mut() as *mut HashMap<u32, Member>;
        unsafe {
            for id in members_ptr.as_ref().unwrap().keys() {
                self.prepare_cancel(id, false);
            }
        }
    }

    ///准备与取消
    pub fn prepare_cancel(&mut self, user_id: &u32, pregare_cancel: bool) {
        let member = self.members.get_mut(user_id).unwrap();
        match pregare_cancel {
            true => member.state = MemberState::Ready,
            false => member.state = MemberState::NotReady,
        }
        //通知其他玩家
        let mut spc = S_PREPARE_CANCEL::new();
        spc.is_succ = true;
        self.prepare_cancel_notice(*user_id, pregare_cancel);
        self.send_2_client(
            ClientCode::PrepareCancel,
            *user_id,
            spc.write_to_bytes().unwrap(),
        );
        if self.check_ready() && self.room_type == RoomType::OneVOneVOneVOneMatch {
            self.start();
        }
    }

    ///房间变更通知
    pub fn room_notice(&mut self) {
        let mut srn = S_ROOM_NOTICE::new();
        srn.owner_id = self.owner_id;
        srn.set_setting(self.setting.borrow().into());
        let bytes = srn.write_to_bytes().unwrap();
        let self_mut_ref = self.get_mut_ref();
        for id in self.members.keys() {
            self_mut_ref.send_2_client(ClientCode::RoomNotice, *id, bytes.clone());
        }
    }

    ///发送表情包
    pub fn emoji(&mut self, user_id: u32, emoji_id: u32) {
        //回给发送人
        let mut sej = S_EMOJI::new();
        sej.is_succ = true;
        self.send_2_client(ClientCode::Emoji, user_id, sej.write_to_bytes().unwrap());

        //推送给房间其他人
        let mut sen = S_EMOJI_NOTICE::new();
        sen.user_id = user_id;
        sen.emoji_id = emoji_id;
        let bytes = sen.write_to_bytes().unwrap();
        let self_mut_ref = self.get_mut_ref();
        for user_id in self.members.keys() {
            self_mut_ref.send_2_client(ClientCode::EmojiNotice, *user_id, bytes.clone());
        }
    }

    ///成员离开推送
    pub fn member_leave_notice(&mut self, notice_type: u8, user_id: &u32, nees_push_self: bool) {
        let mut srmln = S_ROOM_MEMBER_LEAVE_NOTICE::new();
        srmln.set_notice_type(notice_type as u32);
        srmln.set_user_id(*user_id);
        let bytes = srmln.write_to_bytes().unwrap();
        let self_mut_ref = self.get_mut_ref();
        for member_id in self.members.keys() {
            if !nees_push_self && member_id == user_id {
                continue;
            }
            self_mut_ref.send_2_client(ClientCode::MemberLeaveNotice, *member_id, bytes.clone());
        }
    }

    pub fn get_member_index(&self, user_id: u32) -> i32 {
        for i in 0..self.member_index.len() {
            if self.member_index[i] != user_id {
                continue;
            }
            return i as i32;
        }
        -1_i32
    }

    ///推送消息
    pub fn room_add_member_notice(&mut self, user_id: &u32) {
        let mut srmn = S_ROOM_ADD_MEMBER_NOTICE::new();
        srmn.set_index(self.get_member_index(*user_id) as u32);
        let member = self.members.get(user_id);
        if member.is_none() {
            return;
        }
        let mp = member.unwrap().clone().into();
        srmn.set_member(mp);

        let bytes = srmn.write_to_bytes().unwrap();
        let self_mut_ref = self.get_mut_ref();
        if self_mut_ref.get_member_count() > 0 {
            for id in self.members.keys() {
                self_mut_ref.send_2_client(ClientCode::RoomAddMemberNotice, *id, bytes.clone());
            }
        }
    }

    pub fn prepare_cancel_notice(&mut self, user_id: u32, state: bool) {
        let mut spcn = S_PREPARE_CANCEL_NOTICE::new();
        spcn.set_user_id(user_id);
        spcn.set_prepare(state);
        let bytes = spcn.write_to_bytes().unwrap();
        let self_mut_ref = self.get_mut_ref();
        if self.get_member_count() > 0 {
            for id in self.members.keys() {
                self_mut_ref.send_2_client(ClientCode::PrepareCancelNotice, *id, bytes.clone());
            }
        }
    }

    pub fn get_state(&self) -> RoomState {
        self.state
    }

    ///检查准备状态
    pub fn check_ready(&self) -> bool {
        let size = 4;
        let mut index = 0;
        let room_type = self.room_type;
        for member in self.members.values() {
            let res = member.state == MemberState::Ready;
            //如果是房主，并且是自定义房间
            if member.user_id == self.owner_id && room_type == RoomType::OneVOneVOneVOneCustom {
                index += 1;
            }
            if !res {
                continue;
            }
            index += 1;
        }
        index >= size
    }

    ///获得房主ID
    pub fn get_owner_id(&self) -> u32 {
        self.owner_id
    }

    ///获得房间类型
    pub fn get_room_type(&self) -> RoomType {
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
        for i in 0..self.member_index.len() {
            if self.member_index[i] != 0 {
                continue;
            }
            self.member_index[i] = user_id;
            break;
        }

        //不是匹配房就通知其他成员
        if self.room_type != RoomType::OneVOneVOneVOneMatch {
            //返回客户端消息
            let mut sr = S_ROOM::new();
            sr.is_succ = true;
            sr.set_room(self.convert_to_pt());
            self.send_2_client(ClientCode::Room, user_id, sr.write_to_bytes().unwrap());

            //通知房间里其他人
            self.room_add_member_notice(&user_id);
        }
        Ok(self.id)
    }

    //随便获得一个玩家,如果玩家id==0,则代表没有玩家了
    pub fn get_user(&self) -> u32 {
        let mut res = 0;
        for member in self.members.values() {
            if member.is_robot {
                continue;
            }
            let member_id = member.user_id;
            if member_id > res {
                res = member_id;
                break;
            }
        }
        res
    }

    pub fn remove_member_without_push(&mut self, user_id: u32) {
        let res = self.members.get(&user_id);
        if res.is_none() {
            return;
        }
        //删除房间内玩家数据
        self.handler_leave(user_id);
    }

    ///移除玩家
    pub fn remove_member(&mut self, notice_type: u8, user_id: &u32, nees_push_self: bool) {
        let res = self.members.get(user_id);
        if res.is_none() {
            return;
        }

        //通知客户端
        if self.state != RoomState::ChoiceIndex {
            self.member_leave_notice(notice_type, user_id, nees_push_self);
        }
        //删除房间内玩家数据
        self.handler_leave(*user_id);
    }

    ///处理玩家离开
    fn handler_leave(&mut self, user_id: u32) {
        self.members.remove(&user_id);
        let mut index = 0_usize;
        for i in self.member_index.iter() {
            if *i == user_id {
                break;
            }
            index += 1;
        }
        self.member_index[index] = 0;
    }

    ///换队伍
    pub fn change_team(&mut self, user_id: &u32, team_id: &u8) {
        let member = self.get_member_mut(user_id).unwrap();
        member.team_id = *team_id;

        let mut sct = S_CHANGE_TEAM_NOTICE::new();
        sct.set_user_id(*user_id);
        sct.set_team_id(*team_id as u32);
        let bytes = sct.write_to_bytes().unwrap();
        self.send_2_all_client(ClientCode::ChangeTeamNotice, bytes);
    }

    ///T人
    pub fn kick_member(&mut self, user_id: &u32, target_id: &u32) -> Result<(), &str> {
        if self.owner_id != *user_id {
            return Err("不是房主，无法执行该操作");
        }
        if !self.members.contains_key(target_id) {
            return Err("该玩家不在房间内");
        }

        let mut skm = S_KICK_MEMBER::new();
        skm.is_succ = true;
        self.send_2_client(
            ClientCode::KickMember,
            *user_id,
            skm.write_to_bytes().unwrap(),
        );
        //移除玩家
        self.remove_member(MemberLeaveNoticeType::Kicked as u8, target_id, true);

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
        rp.set_room_status(self.state.clone() as u32);
        rp.set_setting(self.setting.borrow().into());
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

    pub fn is_started(&self) -> bool {
        if self.state != RoomState::ChoiceIndex {
            false
        } else {
            true
        }
    }

    ///开始游戏
    pub fn start(&mut self) {
        if self.state == RoomState::ChoiceIndex {
            return;
        }
        self.state = RoomState::ChoiceIndex;
        //通知战斗服务器，游戏开始战斗
        let user_id = self.owner_id;
        let mut rbs = R_B_START::new();
        let res = &*self;
        let rp: RoomPt = res.into();
        rbs.set_room_pt(rp);
        let res = rbs.write_to_bytes();
        if let Err(e) = res {
            error!("{:?}", e);
            return;
        }
        let bytes = res.unwrap();
        self.send_2_server(BattleCode::Start.into_u32(), user_id, bytes);
    }
}
