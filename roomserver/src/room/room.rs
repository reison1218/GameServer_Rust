use crate::battle::battle::BattleData;
use crate::battle::battle_enum::BattleCterState;
use crate::room::character::BattleCharacter;
use crate::room::map_data::TileMap;
use crate::room::member::{Member, MemberState};
use crate::room::room_model::{RoomSetting, RoomType};
use crate::task_timer::{Task, TaskCmd};
use crate::TEMPLATES;
use chrono::{DateTime, Local, Utc};
use log::{error, info, warn};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use protobuf::Message;
use rand::{thread_rng, Rng};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::str::FromStr;
use tools::cmd_code::{ClientCode, GameCode};
use tools::protos::base::{MemberPt, RoomPt, WorldCellPt};
use tools::protos::battle::S_BATTLE_START_NOTICE;
use tools::protos::room::{
    S_CHANGE_TEAM_NOTICE, S_CHOOSE_INDEX_NOTICE, S_CHOOSE_TURN_ORDER_NOTICE, S_EMOJI,
    S_EMOJI_NOTICE, S_KICK_MEMBER, S_PREPARE_CANCEL, S_PREPARE_CANCEL_NOTICE, S_ROOM,
    S_ROOM_ADD_MEMBER_NOTICE, S_ROOM_MEMBER_LEAVE_NOTICE, S_ROOM_NOTICE, S_SKIP_TURN_CHOICE_NOTICE,
    S_START_CHOOSE_INDEX_NOTICE, S_START_NOTICE,
};
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

//最大成员数量
pub const MEMBER_MAX: u8 = 4;

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RoomSettingType {
    None = 0,
    AILevel = 1,
    IsOpenWorldCell = 2,
    TurnLimitTime = 3,
}

impl From<u32> for RoomSettingType {
    fn from(value: u32) -> Self {
        match value {
            1 => RoomSettingType::AILevel,
            2 => RoomSettingType::IsOpenWorldCell,
            3 => RoomSettingType::TurnLimitTime,
            _ => RoomSettingType::None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RoomMemberNoticeType {
    None = 0, //无效
    UpdateMember = 2,
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
    Await = 0,         //等待
    ChoiceTurn = 1,    //选择回合
    ChoiceIndex = 2,   //选择占位
    BattleStarted = 3, //战斗开始
}

///房间结构体，封装房间必要信息
#[derive(Clone)]
pub struct Room {
    id: u32,                              //房间id
    room_type: RoomType,                  //房间类型
    owner_id: u32,                        //房主id
    pub state: RoomState,                 //房间状态
    pub members: HashMap<u32, Member>,    //玩家对应的队伍
    pub member_index: [u32; 4],           //玩家对应的位置
    pub setting: RoomSetting,             //房间设置
    pub battle_data: BattleData,          //战斗相关数据封装
    pub sender: TcpSender,                //sender
    task_sender: crossbeam::Sender<Task>, //任务sender
    time: DateTime<Utc>,                  //房间创建时间
}

impl Room {
    ///构建一个房间的结构体
    pub fn new(
        mut owner: Member,
        room_type: RoomType,
        sender: TcpSender,
        task_sender: crossbeam::Sender<Task>,
    ) -> anyhow::Result<Room> {
        //转换成tilemap数据
        let user_id = owner.user_id;
        let mut str = Local::now().timestamp_subsec_micros().to_string();
        str.push_str(thread_rng().gen_range(1, 999).to_string().as_str());
        let id: u32 = u32::from_str(str.as_str())?;
        let time = Utc::now();
        let mut room = Room {
            id,
            owner_id: user_id,
            members: HashMap::new(),
            member_index: [0; 4],
            state: RoomState::Await,
            setting: RoomSetting::default(),
            battle_data: BattleData::new(task_sender.clone(), sender.clone()),
            room_type,
            sender,
            task_sender,
            time,
        };
        if room.room_type == RoomType::Match {
            let limit_time = TEMPLATES
                .get_constant_ref()
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
        //返回客户端
        let mut sr = S_ROOM::new();
        sr.is_succ = true;
        sr.set_room(room.convert_to_pt());
        room.send_2_client(ClientCode::Room, user_id, sr.write_to_bytes().unwrap());
        Ok(room)
    }

    ///处理结算
    pub unsafe fn handler_summary(&mut self) -> bool {
        if self.state != RoomState::ChoiceIndex && self.state != RoomState::BattleStarted {
            return false;
        }
        let (is_summary, allive_count, summary_data) = self.battle_data.handler_summary();
        if is_summary {
            let bytes = summary_data.unwrap().write_to_bytes().unwrap();
            //发给游戏服同步结算数据
            self.send_2_game(GameCode::Summary, bytes);
        } else if allive_count >= 2 && self.battle_data.tile_map.un_pair_count <= 2 {
            //如果存活玩家>=2并且地图未配对的数量<=2则刷新地图
            //校验是否要刷新地图
            self.battle_data.is_refreshed = true;
            self.refresh_map();
        }
        is_summary
    }

    pub fn send_2_game(&mut self, cmd: GameCode, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd as u32, 0, bytes, true, false);
        self.get_sender_mut().write(bytes);
    }

    ///开始选择占位
    pub fn start_choice_index(&mut self) {
        //刷新地图
        self.state = RoomState::ChoiceIndex;
        self.set_next_turn_index(0);
        //校验下一个是不是为0
        let next_turn_user = self.get_turn_user(None).unwrap();
        if next_turn_user == 0 {
            self.add_next_turn_index();
        }
        info!(
            "choice_turn finish!turn_order:{:?}",
            self.battle_data.turn_orders
        );
        let sbs = S_START_CHOOSE_INDEX_NOTICE::new();
        //如果不是刷新，则需要把cter转换成battle_cter
        if !self.battle_data.is_refreshed {
            self.cter_2_battle_cter();
        }
        let bytes = sbs.write_to_bytes().unwrap();
        for id in self.members.keys() {
            let res = Packet::build_packet_bytes(
                ClientCode::StartChoiceIndexNotice as u32,
                *id,
                bytes.clone(),
                true,
                true,
            );
            self.sender.write(res);
        }
        //开始执行占位逻辑
        self.build_choice_index_task();
    }

    ///刷新地图
    pub fn refresh_map(&mut self) {
        let is_world_cell;
        if self.room_type == RoomType::Match {
            is_world_cell = None;
        } else {
            is_world_cell = Some(self.setting.is_world_tile);
        }
        let res = self.battle_data.reset(is_world_cell);
        if let Err(e) = res {
            error!("{:?}", e);
            return;
        }
        self.start_choice_index();
    }

    pub fn get_member_vec(&self) -> Vec<u32> {
        let mut v = Vec::new();
        for member in self.members.keys() {
            v.push(*member);
        }
        v
    }

    ///判断选择是否能选
    pub fn is_can_choice_turn_now(&self, user_id: u32) -> bool {
        let res = self.get_choice_user(None);
        if let Err(e) = res {
            error!("{:?}", e);
            return false;
        }
        let id = res.unwrap();
        id == user_id
    }

    ///判断选择是否能选
    pub fn is_can_choice_index_now(&self, user_id: u32) -> bool {
        let res = self.get_turn_user(None);
        if let Err(e) = res {
            error!("{:?}", e);
            return false;
        }
        let id = res.unwrap();
        id == user_id
    }

    pub fn set_next_choice_index(&mut self, index: usize) {
        self.battle_data.next_choice_index = index;
    }

    pub fn get_next_choice_index(&self) -> usize {
        self.battle_data.next_choice_index
    }

    pub fn add_next_choice_index(&mut self) {
        self.battle_data.next_choice_index += 1;
        let index = self.battle_data.next_choice_index;
        if index >= MEMBER_MAX as usize {
            return;
        }
        let user_id = self.get_choice_user(Some(index));
        if let Ok(user_id) = user_id {
            if user_id != 0 {
                return;
            }
            self.add_next_choice_index();
        } else {
            warn!("{:?}", user_id.err().unwrap());
        }
    }

    pub fn add_next_turn_index(&mut self) {
        self.battle_data.add_next_turn_index();
        if self.battle_data.next_turn_index == 0 && self.state != RoomState::BattleStarted {
            self.state = RoomState::BattleStarted;
        }
    }

    pub fn insert_turn_orders(&mut self, index: usize, user_id: u32) {
        let size = self.battle_data.turn_orders.len() as isize;
        if index as isize > size - 1 {
            return;
        }
        self.battle_data.turn_orders[index] = user_id;
    }

    pub fn remove_turn_orders(&mut self, index: usize) {
        let size = self.battle_data.turn_orders.len() as isize;
        if index as isize > size - 1 {
            return;
        }
        self.battle_data.turn_orders[index] = 0;
    }

    pub fn remove_choice_order(&mut self, index: usize) {
        let size = self.battle_data.choice_orders.len() as isize;
        if index as isize > size - 1 {
            return;
        }
        self.battle_data.choice_orders[index] = 0;
    }

    pub fn insert_choice_order(&mut self, index: usize, user_id: u32) {
        let size = self.battle_data.choice_orders.len() as isize;
        if index as isize > size - 1 {
            return;
        }
        self.battle_data.choice_orders[index] = user_id;
    }

    pub fn get_next_turn_index(&self) -> usize {
        self.battle_data.next_turn_index
    }

    pub fn set_next_turn_index(&mut self, index: usize) {
        self.battle_data.next_turn_index = index;
    }

    pub fn get_battle_cter_ref(&self, key: &u32) -> Option<&BattleCharacter> {
        self.battle_data.battle_cter.get(key)
    }

    pub fn get_battle_cter_mut_ref(&mut self, key: &u32) -> Option<&mut BattleCharacter> {
        self.battle_data.battle_cter.get_mut(key)
    }

    fn check_index_over(&mut self) -> bool {
        self.state != RoomState::Await
            && self.state != RoomState::ChoiceTurn
            && self.state != RoomState::ChoiceIndex
    }

    ///选择跳过
    pub fn skip_choice_turn(&mut self, user_id: u32) {
        let mut index = self.get_next_choice_index();
        info!(
            "choice skpi_choice_turn user_id:{},index:{},choice_order:{:?}",
            user_id, index, self.battle_data.choice_orders
        );
        self.add_next_choice_index();
        let mut sstcn = S_SKIP_TURN_CHOICE_NOTICE::new();
        sstcn.user_id = user_id;
        let bytes = sstcn.write_to_bytes().unwrap();

        //推送给所有人
        for member_id in self.member_index.iter() {
            let res = Packet::build_packet_bytes(
                ClientCode::SkipChoiceTurnNotice as u32,
                *member_id,
                bytes.clone(),
                true,
                true,
            );
            self.sender.write(res);
        }

        let is_all_choice = self.check_is_all_choice_turn();
        index = self.get_next_choice_index();
        let size = MEMBER_MAX as usize;
        if is_all_choice {
            self.state = RoomState::ChoiceIndex;
            self.set_next_choice_index(0);
        } else if index >= size {
            self.random_choice_turn();
        }
    }

    ///选择占位
    pub fn choice_index(&mut self, user_id: u32, index: u32) {
        let turn_index = self.get_next_turn_index();
        let turn_order = self.battle_data.turn_orders;
        let member = self.get_battle_cter_mut_ref(&user_id);
        if member.is_none() {
            error!("choice_index member is none!user_id:{}", user_id);
            return;
        }
        let member = member.unwrap();

        info!(
            "choice choice_index user_id:{},index:{},turn_order:{:?}",
            user_id, turn_index, turn_order
        );

        //更新角色下标和地图块上面的角色id
        member.cell_index = index as usize;
        let cell = self
            .battle_data
            .tile_map
            .map
            .get_mut(index as usize)
            .unwrap();
        cell.user_id = user_id;
        let mut scln = S_CHOOSE_INDEX_NOTICE::new();
        scln.set_user_id(user_id);
        scln.index = index;
        let bytes = scln.write_to_bytes().unwrap();
        //通知给房间成员
        for id in self.members.keys() {
            let res = Packet::build_packet_bytes(
                ClientCode::ChoiceLoactionNotice as u32,
                *id,
                bytes.clone(),
                true,
                true,
            );
            self.sender.write(res);
        }

        //添加下个turnindex
        self.add_next_turn_index();

        //选完了就进入战斗
        let res = self.check_index_over();
        //都选择完了占位，进入选择回合顺序
        if res {
            self.battle_start();
        } else {
            //没选择完，继续选
            self.build_choice_index_task();
        }
    }

    //检查是否都选了回合顺序
    pub fn check_is_all_choice_turn(&self) -> bool {
        let index = self.get_next_choice_index();
        if index >= MEMBER_MAX as usize {
            for member_id in self.members.keys() {
                if !self.battle_data.turn_orders.contains(member_id) {
                    return false;
                }
            }
        } else {
            return false;
        }
        true
    }

    //给没选都人随机回合顺序
    pub fn random_choice_turn(&mut self) {
        //先选出可以随机的下标
        let mut index_v: Vec<usize> = Vec::new();
        for index in 0..MEMBER_MAX as usize {
            let user_id = self.get_turn_user(Some(index));
            if user_id.is_err() {
                continue;
            }
            let user_id = user_id.unwrap();
            if user_id != 0 {
                continue;
            }
            index_v.push(index);
        }
        let mut rand = rand::thread_rng();
        //如果是最后一个，直接给所有未选的玩家进行随机
        for member_id in self.members.clone().keys() {
            //选过了就跳过
            if self.turn_order_contains(member_id) {
                continue;
            }
            //系统帮忙选
            let remove_index = rand.gen_range(0, index_v.len());
            let index = index_v.get(remove_index).unwrap();
            let turn_order = *index as usize;
            self.choice_turn(*member_id, turn_order, false);
            index_v.remove(remove_index);
        }
        self.state = RoomState::ChoiceIndex;
        self.set_next_turn_index(0);
        let next_turn_user = self.get_turn_user(None).unwrap();
        if next_turn_user == 0 {
            self.add_next_turn_index();
        }
    }

    pub fn turn_order_contains(&self, user_id: &u32) -> bool {
        self.battle_data.turn_orders.contains(user_id)
    }

    ///选择回合
    pub fn choice_turn(&mut self, user_id: u32, order: usize, need_random: bool) {
        if self.state != RoomState::ChoiceTurn {
            return;
        }
        //如果玩家选择的
        self.insert_turn_orders(order, user_id);
        //通知其他玩家
        let mut scron = S_CHOOSE_TURN_ORDER_NOTICE::new();
        scron.user_id = user_id;
        scron.order = order as u32;
        let bytes = scron.write_to_bytes().unwrap();
        for id in self.members.keys() {
            let res = Packet::build_packet_bytes(
                ClientCode::ChoiceRoundOrderNotice as u32,
                *id,
                bytes.clone(),
                true,
                true,
            );
            self.sender.write(res);
        }
        let mut index = self.get_next_choice_index();
        info!(
            "choice choice_turn user_id:{},index:{},choice_order:{:?}",
            user_id, index, self.battle_data.choice_orders
        );
        let size = MEMBER_MAX as usize;
        self.add_next_choice_index();
        index = self.get_next_choice_index();

        let is_all_choice = self.check_is_all_choice_turn();
        if is_all_choice {
            self.start_choice_index();
        } else if !is_all_choice && index >= size && need_random {
            self.random_choice_turn();
        }
        let res = self.check_choice_turn_over();
        //如果都选完了，开始选占位，并发送战斗数据给客户端
        if !res {
            //如果没选完，继续选
            self.build_choice_turn_task();
        }
    }

    pub fn check_choice_turn_over(&mut self) -> bool {
        self.state != RoomState::Await && self.state != RoomState::ChoiceTurn
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, true);
        self.get_sender_mut().write(bytes);
    }

    pub fn get_sender_mut(&mut self) -> &mut TcpSender {
        self.sender.borrow_mut()
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

    ///准备与取消
    pub fn prepare_cancel(&mut self, user_id: &u32, pregare_cancel: bool) {
        let member = self.members.get_mut(user_id).unwrap();
        match pregare_cancel {
            true => member.state = MemberState::Ready as u8,
            false => member.state = MemberState::NotReady as u8,
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
        if self.check_ready() && self.room_type == RoomType::Match {
            self.start();
        }
    }

    ///房间变更通知
    pub fn room_notice(&mut self) {
        let mut srn = S_ROOM_NOTICE::new();
        srn.owner_id = self.owner_id;
        srn.set_setting(self.setting.clone().into());
        let mut packet = Packet::new(ClientCode::RoomNotice as u32, 0, 0);
        packet.set_data_from_vec(srn.write_to_bytes().unwrap());
        packet.set_is_client(true);
        packet.set_is_broad(false);
        for id in self.members.keys() {
            packet.set_user_id(*id);
            self.sender.write(packet.build_server_bytes());
        }
    }

    //战斗通知
    pub fn start_notice(&mut self) {
        let mut ssn = S_START_NOTICE::new();
        ssn.set_room_status(self.state.clone() as u32);
        ssn.set_tile_map_id(self.battle_data.tile_map.id);
        //封装世界块
        for (index, id) in self.battle_data.tile_map.world_cell_map.iter() {
            let mut wcp = WorldCellPt::default();
            wcp.set_index(*index);
            wcp.set_world_cell_id(*id);
            ssn.world_cell.push(wcp);
        }
        //随机出选择的顺序
        let mut random = rand::thread_rng();
        let member_count = self.get_member_count();
        let mut member_v = self.member_index.clone().to_vec();
        member_v.resize(member_count, 0);
        let mut index = 0_u32;

        loop {
            if index > (member_count - 1) as u32 {
                break;
            }
            let rm_index = random.gen_range(0, member_v.len());
            let res = member_v.remove(rm_index);
            if res == 0 {
                continue;
            }
            self.insert_choice_order(index as usize, res);
            index += 1;
        }
        //此一次，所以直接取0下标的值
        self.set_next_choice_index(0);
        ssn.choice_order = self.battle_data.choice_orders.to_vec();
        let bytes = ssn.write_to_bytes().unwrap();
        for id in self.members.keys() {
            let bytes = Packet::build_packet_bytes(
                ClientCode::StartNotice as u32,
                *id,
                bytes.clone(),
                true,
                true,
            );
            self.sender.write(bytes);
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
        for user_id in self.members.keys() {
            let bytes = Packet::build_packet_bytes(
                ClientCode::EmojiNotice as u32,
                *user_id,
                sen.write_to_bytes().unwrap(),
                true,
                true,
            );
            self.sender.write(bytes);
        }
    }

    ///成员离开推送
    pub fn member_leave_notice(&mut self, notice_type: u8, user_id: &u32) {
        let mut srmln = S_ROOM_MEMBER_LEAVE_NOTICE::new();
        srmln.set_notice_type(notice_type as u32);
        srmln.set_user_id(*user_id);
        let mut packet = Packet::new(ClientCode::MemberLeaveNotice as u32, 0, 0);
        packet.set_data_from_vec(srmln.write_to_bytes().unwrap());
        packet.set_is_broad(false);
        packet.set_is_client(true);
        for member_id in self.members.keys() {
            packet.set_user_id(*member_id);
            self.sender.write(packet.build_server_bytes());
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

        let mut packet = Packet::new(ClientCode::RoomAddMemberNotice as u32, 0, 0);
        packet.set_data_from_vec(srmn.write_to_bytes().unwrap());
        packet.set_is_broad(false);
        packet.set_is_client(true);
        if self.get_member_count() > 0 {
            for id in self.members.keys() {
                packet.set_user_id(*id);
                self.sender.write(packet.build_server_bytes());
            }
        }
    }

    pub fn prepare_cancel_notice(&mut self, user_id: u32, state: bool) {
        let mut spcn = S_PREPARE_CANCEL_NOTICE::new();
        spcn.set_user_id(user_id);
        spcn.set_prepare(state);
        let mut packet = Packet::new(ClientCode::PrepareCancelNotice as u32, 0, 0);
        packet.set_data_from_vec(spcn.write_to_bytes().unwrap());
        packet.set_is_broad(false);
        packet.set_is_client(true);
        if self.get_member_count() > 0 {
            for id in self.members.keys() {
                packet.set_user_id(*id);
                self.sender.write(packet.build_server_bytes());
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
        for member in self.members.values() {
            let res = member.state == MemberState::Ready as u8;
            if !res {
                continue;
            }
            index += 1;
        }
        size == index
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

        //返回客户端消息
        let mut sr = S_ROOM::new();
        sr.is_succ = true;
        sr.set_room(self.convert_to_pt());
        self.send_2_client(ClientCode::Room, user_id, sr.write_to_bytes().unwrap());

        //通知房间里其他人
        self.room_add_member_notice(&user_id);
        Ok(self.id)
    }

    ///移除玩家
    pub fn remove_member(&mut self, notice_type: u8, user_id: &u32) {
        let res = self.members.get(user_id);
        if res.is_none() {
            return;
        }

        self.member_leave_notice(notice_type, user_id);
        self.members.remove(user_id);

        for i in 0..self.member_index.len() {
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
            self.room_notice();
        }
        //房间空了就不用处理其他的了
        if self.is_empty() {
            return;
        }
        //处理战斗相关的数据
        self.handler_leave(*user_id);
    }

    fn get_choice_orders(&self) -> &[u32] {
        &self.battle_data.choice_orders[..]
    }

    fn get_turn_orders(&self) -> &[u32] {
        &self.battle_data.turn_orders[..]
    }

    pub fn get_choice_user(&self, _index: Option<usize>) -> anyhow::Result<u32> {
        let index;
        if let Some(_index) = _index {
            index = _index;
        } else {
            index = self.get_next_choice_index();
        }
        let res = self.battle_data.choice_orders.get(index);
        if res.is_none() {
            let str = format!("get_next_choice_user is none for index:{} ", index);
            anyhow::bail!(str)
        }
        let user_id = res.unwrap();
        Ok(*user_id)
    }

    pub fn get_turn_user(&self, _index: Option<usize>) -> anyhow::Result<u32> {
        self.battle_data.get_turn_user(_index)
    }

    ///处理选择回合顺序时候的离开
    fn handler_leave_choice_turn(&mut self, user_id: u32, index: usize) {
        let next_turn_user = self.get_choice_user(None);
        if let Err(e) = next_turn_user {
            error!("{:?}", e);
            return;
        }
        let next_turn_user = next_turn_user.unwrap();
        let member_size = MEMBER_MAX as usize;
        let last_order_user = self.battle_data.choice_orders[member_size - 1];

        //处理正在开始的战斗
        self.remove_choice_order(index);
        //如果当前离开玩家不是当前顺序则退出
        if next_turn_user != user_id {
            return;
        }
        //如果当前玩家正好处于最后一个顺序
        if last_order_user == user_id {
            self.set_next_turn_index(0);
            self.state = RoomState::ChoiceIndex;
            //系统帮忙选回合顺序
            self.random_choice_turn();
            let sbs = S_START_CHOOSE_INDEX_NOTICE::new();
            self.cter_2_battle_cter();
            let bytes = sbs.write_to_bytes().unwrap();
            for id in self.members.keys() {
                let res = Packet::build_packet_bytes(
                    ClientCode::StartChoiceIndexNotice as u32,
                    *id,
                    bytes.clone(),
                    true,
                    true,
                );
                self.sender.write(res);
            }
            //开始执行占位逻辑
            self.build_choice_index_task();
        } else {
            //不是最后一个就轮到下一个
            self.add_next_choice_index();
            self.build_choice_turn_task();
        }
    }

    ///处理选择占位时候的离开
    fn handler_leave_choice_index(&mut self, user_id: u32, index: usize) {
        let next_turn_user = self.get_turn_user(None);
        if let Err(e) = next_turn_user {
            error!("{:?}", e);
            return;
        }
        let next_turn_user = next_turn_user.unwrap();
        let member_size = MEMBER_MAX as usize;

        //去掉地图块上的玩家id
        let cell = self.battle_data.tile_map.map.get_mut(index);
        if let Some(cell) = cell {
            cell.user_id = 0;
        }

        let last_order_user = self.battle_data.turn_orders[member_size - 1];
        self.remove_turn_orders(index);

        //移除战斗角色
        self.battle_data.battle_cter.remove(&user_id);
        self.battle_data.tile_map.remove_user(user_id);
        //如果当前离开的玩家不是当前顺序就退出
        if next_turn_user != user_id {
            return;
        }
        //如果当前玩家正好处于最后一个顺序
        if last_order_user == user_id {
            self.state = RoomState::BattleStarted;
            self.set_next_turn_index(0);
            let next_turn_user = self.get_turn_user(None).unwrap();
            if next_turn_user == 0 {
                self.add_next_turn_index();
            }
            self.battle_start();
        } else {
            //不是最后一个就轮到下一个
            self.add_next_turn_index();
            self.build_choice_index_task();
        }
    }

    ///处理选择战斗回合时候的离开
    fn handler_leave_battle_turn(&mut self, user_id: u32, index: usize) {
        let next_turn_user = self.get_turn_user(None);
        if let Err(e) = next_turn_user {
            error!("{:?}", e);
            return;
        }
        let next_turn_user = next_turn_user.unwrap();
        //移除顺位数据
        self.remove_turn_orders(index);
        //移除玩家战斗数据
        self.battle_data.battle_cter.remove(&user_id);
        self.battle_data.tile_map.remove_user(user_id);
        //如果当前离开的玩家不是当前顺序就退出
        if next_turn_user != user_id {
            return;
        }
        self.battle_data.next_turn();
    }

    ///处理玩家离开
    fn handler_leave(&mut self, user_id: u32) {
        let mut chocie_index = 0;
        let mut turn_index = 0;
        //找出离开玩家的选择下标
        for i in self.get_choice_orders() {
            if i == &user_id {
                break;
            }
            chocie_index += 1;
        }

        //找出离开玩家的回合下标
        for i in self.get_turn_orders() {
            if i == &user_id {
                break;
            }
            turn_index += 1;
        }
        if self.state == RoomState::ChoiceTurn {
            self.handler_leave_choice_turn(user_id, chocie_index);
        } else if self.state == RoomState::ChoiceIndex {
            self.handler_leave_choice_index(user_id, chocie_index);
        } else if self.state == RoomState::BattleStarted {
            self.handler_leave_battle_turn(user_id, turn_index);
        }

        //处理结算
        unsafe {
            self.handler_summary();
        }
    }

    ///换队伍
    pub fn change_team(&mut self, user_id: &u32, team_id: &u8) {
        let member = self.get_member_mut(user_id).unwrap();
        member.team_id = *team_id;

        let mut sct = S_CHANGE_TEAM_NOTICE::new();
        sct.set_user_id(*user_id);
        sct.set_team_id(*team_id as u32);
        let bytes = sct.write_to_bytes().unwrap();
        let members = self.members.clone();
        for member_id in members.keys() {
            self.send_2_client(ClientCode::ChangeTeamNotice, *member_id, bytes.clone());
        }
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
        self.remove_member(MemberLeaveNoticeType::Kicked as u8, target_id);

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
        rp.set_setting(self.setting.clone().into());
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
        let battle_cter = self.battle_data.battle_cter.get_mut(user_id).unwrap();
        battle_cter.target_id = *target_id;
        Ok(())
    }

    pub fn cter_2_battle_cter(&mut self) {
        for member in self.members.values_mut() {
            let battle_cter = BattleCharacter::init(&member.chose_cter);
            match battle_cter {
                Ok(b_cter) => {
                    self.battle_data.battle_cter.insert(member.user_id, b_cter);
                }
                Err(_) => {
                    return;
                }
            }
        }
    }

    pub fn is_started(&self) -> bool {
        if self.state != RoomState::BattleStarted {
            false
        } else {
            true
        }
    }

    ///开始游戏
    pub fn start(&mut self) {
        //生成地图
        let res = self.generate_map();
        if let Err(e) = res {
            error!("{:?}", e);
            return;
        }
        self.battle_data.tile_map = res.unwrap();
        self.battle_data.turn_limit_time = self.setting.turn_limit_time as u64;
        //改变房间状态
        self.state = RoomState::ChoiceTurn;
        //下发通知
        self.start_notice();
        //创建choice_turn定时器任务
        self.build_choice_turn_task();
    }

    ///生成地图
    pub fn generate_map(&self) -> anyhow::Result<TileMap> {
        let member_count = self.members.len() as u32;
        let is_world_cell;
        if self.room_type == RoomType::Match {
            is_world_cell = None;
        } else {
            is_world_cell = Some(self.setting.is_world_tile);
        }
        let tmd = TileMap::init(member_count, is_world_cell)?;
        Ok(tmd)
    }

    pub fn build_choice_turn_task(&self) {
        let user_id = self.get_choice_user(None);
        if let Err(_) = user_id {
            return;
        }
        let user_id = user_id.unwrap();

        //没选择完，继续选
        let time_limit = TEMPLATES.get_constant_ref().temps.get("choice_turn_time");
        let mut task = Task::default();
        if let Some(time) = time_limit {
            let time = u64::from_str(time.value.as_str());
            match time {
                Ok(time) => {
                    task.delay = time + 500;
                }
                Err(e) => {
                    task.delay = 5000_u64;
                    error!("{:?}", e);
                }
            }
        } else {
            task.delay = 5000_u64;
            warn!("the choice_turn_time of Constant config is None!pls check!");
        }
        task.cmd = TaskCmd::ChoiceTurnOrder as u16;

        let mut map = serde_json::Map::new();
        map.insert("user_id".to_owned(), serde_json::Value::from(user_id));
        task.data = serde_json::Value::from(map);
        let res = self.task_sender.send(task);
        if res.is_err() {
            error!("{:?}", res.err().unwrap());
        }
    }

    ///战斗开始
    pub fn battle_start(&mut self) {
        //判断是否有世界块,有的话，
        if !self.battle_data.tile_map.world_cell_map.is_empty() && !self.battle_data.is_refreshed {
            for world_cell_id in self.battle_data.tile_map.world_cell_map.values() {
                let world_cell_temp = TEMPLATES.get_world_cell_ref().temps.get(world_cell_id);
                if world_cell_temp.is_none() {
                    error!("world_cell_temp is None! world_cell_id:{}", world_cell_id);
                    continue;
                }
                let world_cell_temp = world_cell_temp.unwrap();
                for buff_id in world_cell_temp.buff.iter() {
                    for (_, battle_cter) in self.battle_data.battle_cter.iter_mut() {
                        battle_cter.add_buff(*buff_id);
                    }
                }
            }
        }
        let mut sbsn = S_BATTLE_START_NOTICE::new();
        for battle_cter in self.battle_data.battle_cter.values() {
            let cter_pt = battle_cter.convert_to_battle_cter();
            sbsn.battle_cters.push(cter_pt);
        }
        let res = sbsn.write_to_bytes();
        if let Err(e) = res {
            error!("{:?}", e);
            return;
        }
        let bytes = res.unwrap();
        let members = self.members.clone();
        for member_id in members.keys() {
            self.send_2_client(ClientCode::BattleStartedNotice, *member_id, bytes.clone());
        }
        self.battle_data.send_battle_turn_notice();
        self.battle_data.build_battle_turn_task();
    }

    ///创建选择下标定时任务
    pub fn build_choice_index_task(&self) {
        let user_id = self.get_turn_user(None);
        if let Err(e) = user_id {
            error!("{:?}", e);
            return;
        }
        let user_id = user_id.unwrap();
        let time_limit = TEMPLATES.get_constant_ref().temps.get("choice_index_time");
        let mut task = Task::default();
        if let Some(time) = time_limit {
            let time = u64::from_str(time.value.as_str());
            match time {
                Ok(time) => {
                    task.delay = time + 500;
                }
                Err(e) => {
                    task.delay = 5000_u64;
                    error!("{:?}", e);
                }
            }
        } else {
            task.delay = 5000_u64;
            warn!("the choice_index_time of Constant config is None!pls check!");
        }
        task.cmd = TaskCmd::ChoiceIndex as u16;

        let mut map = serde_json::Map::new();
        map.insert("user_id".to_owned(), serde_json::Value::from(user_id));
        task.data = serde_json::Value::from(map);
        let res = self.task_sender.send(task);
        if res.is_err() {
            error!("{:?}", res.err().unwrap());
        }
    }
}
