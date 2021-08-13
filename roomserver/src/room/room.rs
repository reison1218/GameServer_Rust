use crate::room::member::{Member, MemberState};
use crate::room::room_model::{RoomSetting, RoomType};
use crate::task_timer::Task;
use crate::ROOM_ID;
use chrono::{DateTime, Local, Utc};
use crossbeam::channel::Sender;
use log::{error, info, warn};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use protobuf::Message;
use rand::Rng;
use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};
use tools::cmd_code::{BattleCode, ClientCode, GameCode};
use tools::macros::GetMutRef;
use tools::protos::base::{MemberPt, RoomPt};
use tools::protos::room::{
    S_CHANGE_TEAM_NOTICE, S_CONFIRM_INTO_ROOM_NOTICE, S_EMOJI, S_EMOJI_NOTICE, S_KICK_MEMBER,
    S_MATCH_SUCCESS_NOTICE, S_PREPARE_CANCEL, S_PREPARE_CANCEL_NOTICE, S_PUNISH_MATCH_NOTICE,
    S_ROOM, S_ROOM_ADD_MEMBER_NOTICE, S_ROOM_MEMBER_LEAVE_NOTICE, S_ROOM_NOTICE,
};
use tools::protos::server_protocol::{B_R_G_PUNISH_MATCH, R_B_START};
use tools::tcp_message_io::TcpHandler;
use tools::util::packet::Packet;

///最大成员数量
pub const MEMBER_MAX: usize = 4;

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RoomSettingType {
    None = 0,
    SeasonId = 1,
    TurnLimitTime = 2,
    AILevel = 3,
}

impl From<u32> for RoomSettingType {
    fn from(value: u32) -> Self {
        match value {
            1 => RoomSettingType::SeasonId,
            2 => RoomSettingType::TurnLimitTime,
            3 => RoomSettingType::AILevel,
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
    AwaitReady = 1,   //等待
    ChoiceIndex = 2,  //选择占位
}

///房间结构体，封装房间必要信息
#[derive(Clone)]
pub struct Room {
    id: u32,                             //房间id
    room_type: RoomType,                 //房间类型
    owner_id: u32,                       //房主id
    pub state: RoomState,                //房间状态
    pub members: HashMap<u32, Member>,   //玩家对应的队伍
    pub member_index: [u32; MEMBER_MAX], //玩家对应的位置
    pub robots: HashSet<u32>,            //机器人
    pub setting: RoomSetting,            //房间设置
    pub tcp_handler: TcpHandler,         //tcpsender
    task_sender: Sender<Task>,           //任务sender
    time: DateTime<Utc>,                 //房间创建时间
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
            let mp = member.into();
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
        sender: TcpHandler,
        task_sender: Sender<Task>,
    ) -> anyhow::Result<Room> {
        //转换成tilemap数据
        let user_id = owner.user_id;
        let id: u32 = create_room_id();
        let time = Utc::now();
        let mut room_state = RoomState::AwaitReady;
        if room_type == RoomType::OneVOneVOneVOneMatch {
            room_state = RoomState::AwaitConfirm;
            owner.state = MemberState::AwaitConfirm;
        }
        let mut room = Room {
            id,
            owner_id: user_id,
            members: HashMap::new(),
            member_index: [0; MEMBER_MAX],
            robots: HashSet::new(),
            state: room_state,
            setting: RoomSetting::default(),
            room_type,
            tcp_handler: sender,
            task_sender,
            time,
        };
        let mut size = room.members.len() as u8;
        size += 1;
        owner.team_id = size;
        owner.join_time = Local::now().timestamp_millis() as u64;
        room.members.insert(user_id, owner);
        room.member_index[0] = user_id;
        info!(
            "创建房间,room_type:{:?},room_id:{},user_id:{}",
            room_type, id, user_id
        );
        Ok(room)
    }

    pub fn is_all_robot(&self) -> bool {
        for member in self.members.values() {
            if member.robot_temp_id == 0 {
                return false;
            }
        }
        true
    }

    pub fn check_need_rm_room(&self) -> bool {
        let room_type = self.room_type;
        let state = self.state;
        if self.is_empty() {
            return true;
        }
        if self.is_all_robot() {
            return true;
        }
        match room_type {
            RoomType::OneVOneVOneVOneCustom => {
                if self.get_owner_id() == 0 && state != RoomState::ChoiceIndex {
                    return true;
                }
            }
            RoomType::OneVOneVOneVOneMatch => {
                if state == RoomState::ChoiceIndex && self.members.len() == 1 {
                    return true;
                }
            }
            _ => {
                return false;
            }
        }
        false
    }

    ///离开房间检查是否需要添加惩罚
    pub fn check_punish_for_leave(&mut self, user_id: u32) {
        if self.room_type != RoomType::OneVOneVOneVOneMatch {
            return;
        }
        //判断是否需要重制
        let member = self.members.get_mut(&user_id);
        if let None = member {
            return;
        }
        let member = member.unwrap();
        member.punish_match.add_punish();
        let pm = member.punish_match;
        //同步到游戏服
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
        //推送给客户端
        let mut proto = S_PUNISH_MATCH_NOTICE::new();
        proto.set_user_id(user_id);
        proto.set_punish_match(pm.into());
        let bytes = proto.write_to_bytes();
        match bytes {
            Ok(bytes) => {
                self.send_2_client(ClientCode::PunishPatchPush, user_id, bytes);
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
        let tcp = self.tcp_handler.borrow();
        let endpoint = tcp.endpoint;
        tcp.node_handler.network().send(endpoint, bytes.as_slice());
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
        if member.robot_temp_id > 0 {
            return;
        }
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, true);
        let tcp = self.tcp_handler.borrow();
        let endpoint = tcp.endpoint;
        tcp.node_handler.network().send(endpoint, bytes.as_slice());
    }

    pub fn push_match_success(&mut self) {
        let mut smsn = S_MATCH_SUCCESS_NOTICE::new();
        let mut confirm_count = 0;
        for member in self.members.values() {
            //如果是机器人，跳过
            if member.robot_temp_id > 0 {
                continue;
            }
            if member.state == MemberState::AwaitConfirm {
                continue;
            }
            confirm_count += 1;
        }
        let mut member_id;
        for member in self.members.values() {
            //如果是机器人，跳过
            if member.robot_temp_id > 0 {
                continue;
            }
            if member.state != MemberState::AwaitConfirm {
                continue;
            }
            member_id = member.user_id;
            smsn.set_count(confirm_count);
            let bytes = smsn.write_to_bytes().unwrap();
            let datas = Packet::build_packet_bytes(
                ClientCode::MatchSuccessNotice.into_u32(),
                member_id,
                bytes.clone(),
                true,
                true,
            );
            let tcp = self.tcp_handler.borrow();
            let endpoint = tcp.endpoint;
            tcp.node_handler.network().send(endpoint, datas.as_slice());
        }
    }

    pub fn send_2_all_client(&mut self, cmd: ClientCode, bytes: Vec<u8>) {
        let mut user_id;
        for member in self.members.values() {
            user_id = member.user_id;
            //如果是机器人，则返回，不发送
            if member.robot_temp_id > 0 {
                continue;
            }
            let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes.clone(), true, true);
            let tcp = self.tcp_handler.borrow();
            let endpoint = tcp.endpoint;
            tcp.node_handler.network().send(endpoint, bytes.as_slice());
        }
    }

    ///检查角色
    pub fn check_character(&self, user_id: u32, cter_id: u32) -> anyhow::Result<()> {
        for cter in self.members.values() {
            if cter.user_id == user_id {
                continue;
            }
            if cter.chose_cter.cter_temp_id == cter_id && cter_id != 0 {
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
        let member_state = member.state;
        match pregare_cancel {
            true => member.state = MemberState::Ready,
            false => member.state = MemberState::NotReady,
        }
        //如果状态改变了通知其他玩家
        if member_state != member.state {
            let mut spc = S_PREPARE_CANCEL::new();
            spc.is_succ = true;
            self.prepare_cancel_notice(*user_id, pregare_cancel);
            self.send_2_client(
                ClientCode::PrepareCancel,
                *user_id,
                spc.write_to_bytes().unwrap(),
            );
        }

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
        let member = self.members.get(user_id);
        if member.is_none() {
            return;
        }
        let mut srmn = S_ROOM_ADD_MEMBER_NOTICE::new();
        srmn.set_index(self.get_member_index(*user_id) as u32);
        let mp = member.unwrap().into();
        srmn.set_member(mp);

        let bytes = srmn.write_to_bytes().unwrap();
        let mut need_push_v = vec![];
        for (_, member) in self.members.iter() {
            if member.state == MemberState::AwaitConfirm {
                continue;
            }
            need_push_v.push(member.user_id);
        }
        if need_push_v.is_empty() {
            return;
        }
        for id in need_push_v {
            self.send_2_client(ClientCode::RoomAddMemberNotice, id, bytes.clone());
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
    pub fn check_ready(&mut self) -> bool {
        let mut index = 0;
        let mut size = self.members.len();
        let owner_id = self.owner_id;
        if self.room_type == RoomType::OneVOneVOneVOneMatch {
            size = MEMBER_MAX;
        }
        for member in self.members.values_mut() {
            if owner_id == member.user_id && self.room_type == RoomType::OneVOneVOneVOneCustom {
                if member.chose_cter.cter_temp_id == 0 {
                    warn!(
                        "check_ready: this player has not choose character yet!user_id:{}",
                        member.get_user_id()
                    );
                    return false;
                }

                let cter_temp = crate::TEMPLATES
                    .character_temp_mgr()
                    .temps
                    .get(&member.chose_cter.cter_temp_id)
                    .unwrap();

                //校验玩家是否选了技能
                if member.chose_cter.skills.len() < cter_temp.usable_skill_count as usize {
                    warn!(
                        "check_ready: this player has not choose character'skill yet!user_id:{}",
                        member.get_user_id()
                    );
                    false;
                }
                if member.chose_cter.skills.len() == 0 {
                    return false;
                }
                member.state = MemberState::Ready;
            }
            let res = member.state == MemberState::Ready;

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
    pub fn add_member(&mut self, mut member: Member, index: Option<usize>) -> anyhow::Result<u32> {
        let mut size = self.members.len() as u8;
        let user_id = member.user_id;
        size += 1;
        if member.team_id == 0 {
            member.team_id = size;
        }
        member.join_time = Local::now().timestamp_millis() as u64;
        self.members.insert(user_id, member);
        match index {
            Some(index) => self.member_index[index] = user_id,
            None => {
                for i in 0..self.member_index.len() {
                    if self.member_index[i] != 0 {
                        continue;
                    }
                    self.member_index[i] = user_id;
                    break;
                }
            }
        }

        //不是匹配房就通知其他成员
        if self.room_type != RoomType::OneVOneVOneVOneMatch {
            self.notice_new_member(user_id);
        }
        Ok(self.id)
    }

    pub fn notice_confirm_count(&mut self, user_id: u32) {
        let member = self.get_member_mut(&user_id).unwrap();
        member.state = MemberState::NotReady;
        let mut count = 0;
        let mut need_push_vec = vec![];
        for member in self.members.values() {
            need_push_vec.push(member.user_id);
            if member.state == MemberState::AwaitConfirm {
                continue;
            }
            count += 1;
        }

        //推送给待确认进入房间人数
        for member_id in need_push_vec {
            let mut scirn = S_CONFIRM_INTO_ROOM_NOTICE::new();
            scirn.set_count(count);
            let bytes = scirn.write_to_bytes().unwrap();
            self.send_2_client(ClientCode::ConfirmIntoRoomNotice, member_id, bytes);
        }
    }

    pub fn notice_new_member(&mut self, user_id: u32) {
        //返回客户端消息
        let mut sr = S_ROOM::new();
        sr.is_succ = true;
        sr.set_room(self.convert_to_pt());
        self.send_2_all_client(ClientCode::Room, sr.write_to_bytes().unwrap());

        //通知房间里其他人
        self.room_add_member_notice(&user_id);
    }

    //随便获得一个玩家,如果玩家id==0,则代表没有玩家了
    pub fn get_user(&self) -> u32 {
        let mut res = 0;
        for member in self.members.values() {
            if member.robot_temp_id > 0 {
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
        if self.get_owner_id() == user_id {
            self.owner_id = 0;
            if self.room_type == RoomType::OneVOneVOneVOneMatch {
                for &id in self.members.keys() {
                    if id != 0 {
                        self.owner_id = id;
                    }
                }
            }
        }
        let (index, _) = self
            .member_index
            .iter()
            .enumerate()
            .find(|(_, &id)| id == user_id)
            .unwrap();
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
                let mp = member.into();
                rp.members.push(mp);
            } else {
                let mp = MemberPt::new();
                rp.members.push(mp);
            }
        }
        rp
    }

    pub fn is_started(&self) -> bool {
        if self.state == RoomState::ChoiceIndex {
            true
        } else {
            false
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
        info!(
            "战斗开始，向战斗服发送BattleCode::Start，room_type:{:?},room_id:{},user_id:{}",
            self.room_type,
            self.get_room_id(),
            user_id
        );
    }
}

pub fn create_room_id() -> u32 {
    unsafe {
        let size = ROOM_ID.len() - 1;
        let mut rand = rand::thread_rng();
        let index = rand.gen_range(0..=size);
        let res = ROOM_ID.remove(index);
        res
    }
}

pub fn recycle_room_id(room_id: u32) {
    unsafe {
        ROOM_ID.push(room_id);
    }
}
