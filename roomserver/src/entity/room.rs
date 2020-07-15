use crate::entity::character::BattleCharacter;
use crate::entity::map_data::CellType;
use crate::entity::map_data::TileMap;
use crate::entity::member::{Member, MemberState};
use crate::entity::room_model::RoomSetting;
use crate::task_timer::{Task, TaskCmd};
use crate::TEMPLATES;
use chrono::{DateTime, Local, Utc};
use log::{error, info, warn};
use protobuf::Message;
use rand::{thread_rng, Rng};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::str::FromStr;
use tools::cmd_code::ClientCode;
use tools::protos::base::{MemberPt, RoomPt, WorldCellPt};
use tools::protos::room::{
    S_BATTLE_CHARACTER_NOTICE, S_CHANGE_TEAM, S_CHOOSE_INDEX_NOTICE, S_CHOOSE_TURN_ORDER_NOTICE,
    S_EMOJI, S_EMOJI_NOTICE, S_KICK_MEMBER, S_PREPARE_CANCEL, S_ROOM, S_ROOM_MEMBER_LEAVE_NOTICE,
    S_ROOM_MEMBER_NOTICE, S_ROOM_NOTICE, S_SKIP_TURN_CHOICE_NOTICE, S_START_NOTICE,
};
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

//最大成员数量
pub const MEMBER_MAX: u8 = 4;

#[derive(Clone, Debug, PartialEq)]
pub enum RoomMemberNoticeType {
    AddMember = 1,
    UpdateMember = 2,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MemberLeaveNoticeType {
    Leave = 1,  //自己离开
    Kicked = 2, //被T
}

#[derive(Clone, Debug, PartialEq)]
pub enum RoomState {
    Await = 0,         //等待
    ChoiceTurn = 1,    //选择回合
    ChoiceIndex = 2,   //选择占位
    BattleStarted = 3, //战斗开始
}

///回合行为类型
#[derive(Clone, Debug, PartialEq)]
enum ActionType {
    Attack = 1,  //普通攻击
    UseItem = 2, //使用道具
    Skip = 3,    //跳过turn
    Open = 4,    //翻块
    Skill = 5,   //使用技能
}

///行动单位
#[derive(Clone, Debug, Default)]
pub struct ActionUnit {
    pub team_id: u32,
    pub user_id: u32,
    pub turn_index: u32,
    pub actions: Vec<Action>,
}

#[derive(Clone, Debug, Default)]
pub struct Action {
    action_type: u8,
    action_value: u32,
}

///房间战斗数据封装
#[derive(Clone, Debug, Default)]
pub struct BattleData {
    pub choice_orders: Vec<u32>,                    //选择顺序里面放玩家id
    pub next_choice_index: usize,                   //下一个选择的下标
    pub next_turn_index: usize,                     //下个turn的下标
    pub turn_action: ActionUnit,                    //当前回合数据单元封装
    pub turn_orders: Vec<u32>,                      //turn行动队列，里面放玩家id
    pub battle_cter: HashMap<u32, BattleCharacter>, //角色战斗数据
}

///房间结构体，封装房间必要信息
#[derive(Clone, Debug)]
pub struct Room {
    id: u32,                              //房间id
    room_type: u8,                        //房间类型
    owner_id: u32,                        //房主id
    state: RoomState,                     //房间状态
    pub tile_map: TileMap,                //地图数据
    pub members: HashMap<u32, Member>,    //玩家对应的队伍
    pub member_index: Vec<u32>,           //玩家对应的位置
    pub setting: RoomSetting,             //房间设置
    pub battle_data: BattleData,          //战斗相关数据封装
    sender: TcpSender,                    //sender
    task_sender: crossbeam::Sender<Task>, //任务sender
    time: DateTime<Utc>,                  //房间创建时间
}

impl Room {
    ///构建一个房间的结构体
    pub fn new(
        mut owner: Member,
        room_type: u8,
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
            id: id,
            owner_id: user_id,
            tile_map: TileMap::default(),
            members: HashMap::new(),
            member_index: Vec::new(),
            state: RoomState::Await,
            setting: RoomSetting::default(),
            battle_data: BattleData::default(),
            room_type,
            sender,
            task_sender,
            time,
        };

        let mut size = room.members.len() as u8;
        size += 1;
        owner.team_id = size;
        owner.join_time = Local::now().timestamp_millis() as u64;
        room.members.insert(user_id, owner);
        room.member_index.push(user_id);
        //返回客户端
        let mut sr = S_ROOM::new();
        sr.is_succ = true;
        sr.set_room(room.convert_to_pt());
        room.send_2_client(ClientCode::Room, user_id, sr.write_to_bytes().unwrap());
        Ok(room)
    }

    pub fn check_choice_index(&self, index: usize) -> bool {
        let res = self.tile_map.map.get(index);
        if let Some(cell) = res {
            if cell.id > CellType::Valid as u32 {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    ///判断选择是否能选
    pub fn is_can_choice_now(&self, step: RoomState, user_id: u32) -> bool {
        match step {
            RoomState::ChoiceIndex => self.get_next_choice_user() == user_id,
            RoomState::ChoiceTurn => self.get_next_choice_user() == user_id,
            _ => false,
        }
    }

    pub fn set_next_choice_index(&mut self, index: usize) {
        self.battle_data.next_choice_index = index;
    }

    pub fn get_next_choice_index(&self) -> usize {
        self.battle_data.next_choice_index
    }

    pub fn get_turn_orders(&self) -> &[u32] {
        &self.battle_data.turn_orders[..]
    }

    pub fn insert_turn_orders(&mut self, index: usize, user_id: u32) {
        let size = self.battle_data.turn_orders.len() as isize;
        if index as isize >= size - 1 {
            self.battle_data.turn_orders.push(user_id);
        } else {
            self.battle_data.turn_orders.remove(index);
            self.battle_data.turn_orders.insert(index, user_id);
        }
    }

    pub fn remove_turn_orders(&mut self, index: usize) {
        let size = self.battle_data.turn_orders.len() as isize;
        if index as isize > size - 1 {
            return;
        }
        self.battle_data.turn_orders.remove(index);
    }

    pub fn get_choice_orders(&self) -> &[u32] {
        &self.battle_data.choice_orders[..]
    }

    pub fn remove_choice_order(&mut self, index: usize) {
        let size = self.battle_data.turn_orders.len() as isize;
        if index as isize > size - 1 {
            return;
        }
        self.battle_data.choice_orders.remove(index as usize);
    }

    pub fn insert_choice_order(&mut self, index: usize, user_id: u32) {
        let size = self.battle_data.choice_orders.len() as isize;
        if index as isize >= size - 1 {
            self.battle_data.choice_orders.push(user_id);
        } else {
            self.battle_data.choice_orders.remove(index);
            self.battle_data.choice_orders.insert(index, user_id);
        }
    }

    pub fn get_next_turn_index(&self) -> usize {
        self.battle_data.next_turn_index
    }

    pub fn set_next_turn_index(&mut self, index: usize) {
        self.battle_data.next_turn_index = index;
    }

    pub fn insert_battle_cter(&mut self, key: u32, value: BattleCharacter) {
        self.battle_data.battle_cter.insert(key, value);
    }

    pub fn get_battle_cter_ref(&self, key: &u32) -> Option<&BattleCharacter> {
        self.battle_data.battle_cter.get(key)
    }

    pub fn get_battle_cter_mut_ref(&mut self, key: &u32) -> Option<&mut BattleCharacter> {
        self.battle_data.battle_cter.get_mut(key)
    }

    pub fn is_battle_do_nothing(&self) -> bool {
        self.battle_data.turn_action.actions.is_empty()
    }

    fn check_index_over(&mut self) -> bool {
        self.state != RoomState::Await
            && self.state != RoomState::ChoiceTurn
            && self.state != RoomState::ChoiceIndex
    }

    ///选择跳过
    pub fn skip_choice_turn(&mut self, user_id: u32) {
        let index = self.get_next_choice_index();
        if index < self.get_member_count() - 1 {
            self.set_next_choice_index(index + 1);
        }
        let mut sstcn = S_SKIP_TURN_CHOICE_NOTICE::new();
        sstcn.user_id = user_id;
        let bytes = sstcn.write_to_bytes().unwrap();

        //推送给所有人
        for member_id in self.member_index.iter() {
            let res = Packet::build_packet_bytes(
                ClientCode::SkipTurnNotice as u32,
                *member_id,
                bytes.clone(),
                true,
                true,
            );
            self.sender.write(res);
        }
    }

    ///选择占位
    pub fn choice_index(&mut self, user_id: u32, index: u32) {
        //玩家手动选的
        let member = self.get_battle_cter_mut_ref(&user_id).unwrap();
        member.cell_index = index;
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

        let index = self.get_next_choice_index();
        //校验是否选完了
        if index >= self.get_member_count() - 1 {
            self.state = RoomState::BattleStarted;
        } else {
            self.set_next_choice_index(index + 1);
        }

        //选完了就进入战斗
        let res = self.check_index_over();
        //都选择完了占位，进入选择回合顺序
        if res {
            self.build_choice_turn_task();
        } else {
            //没选择完，继续选
            self.build_choice_index_task();
        }
    }

    ///选择回合
    pub fn choice_turn(&mut self, user_id: u32, order: usize) {
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
        let index = self.get_next_choice_index();
        //判断是否最后一个选的,是就进入选择占位状态
        if index >= self.get_member_count() - 1 {
            self.state = RoomState::ChoiceIndex;
            self.set_next_choice_index(0);
        } else {
            self.set_next_choice_index(index + 1);
        }
        let res = self.check_choice_turn_over();
        //如果都选完了，开始选占位，并发送战斗数据给客户端
        if res {
            let mut sbs = S_BATTLE_CHARACTER_NOTICE::new();
            self.cter_2_battle_cter();
            for battle_cter in self.battle_data.battle_cter.values() {
                sbs.battle_cters.push(battle_cter.convert_to_battle_cter());
            }

            let bytes = sbs.write_to_bytes().unwrap();
            for id in self.members.keys() {
                let res = Packet::build_packet_bytes(
                    ClientCode::BattleStartNotice as u32,
                    *id,
                    bytes.clone(),
                    true,
                    true,
                );
                self.sender.write(res);
            }
            //开始执行占位逻辑
            self.build_choice_index_task();
        //此处应该加上第一回合限制时间定时器
        } else {
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

    pub fn set_sender(&mut self, sender: TcpSender) {
        self.sender = sender;
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
        self.send_2_client(
            ClientCode::PrepareCancel,
            *user_id,
            spc.write_to_bytes().unwrap(),
        );
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
        ssn.set_tile_map_id(self.tile_map.id);
        let cell_v = self.tile_map.get_cells();
        ssn.cells = cell_v;
        //封装世界块
        for (index, id) in self.tile_map.world_cell_map.iter() {
            let mut wcp = WorldCellPt::default();
            wcp.set_index(*index);
            wcp.set_index(*id);
            ssn.world_cell.push(wcp);
        }
        //随机出选择的顺序
        let mut random = rand::thread_rng();
        let mut member_v = self.member_index.clone();
        let mut index = 0_u32;
        loop {
            if index >= (self.member_index.len() - 1) as u32 {
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
        ssn.choice_order = self.get_choice_orders().to_vec();
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
        self.build_choice_turn_task();
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
            if self.member_index.get(i).unwrap() != &user_id {
                continue;
            }
            return i as i32;
        }
        -1_i32
    }

    ///推送消息
    pub fn room_member_notice(&mut self, notice_type: u8, user_id: &u32) {
        let mut srmn = S_ROOM_MEMBER_NOTICE::new();
        srmn.set_notice_type(notice_type as u32);
        srmn.set_index(self.get_member_index(*user_id) as u32);
        let member = self.members.get(user_id);
        if member.is_none() {
            return;
        }
        let mp = member.unwrap().clone().into();
        srmn.set_member(mp);

        let mut packet = Packet::new(ClientCode::RoomMemberNotice as u32, 0, 0);
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

    pub fn get_state(&self) -> &RoomState {
        &self.state
    }

    pub fn set_status(&mut self, status: RoomState) {
        self.state = status;
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
        self.member_index.push(user_id);

        //返回客户端消息
        let mut sr = S_ROOM::new();
        sr.is_succ = true;
        sr.set_room(self.convert_to_pt());
        self.send_2_client(ClientCode::Room, user_id, sr.write_to_bytes().unwrap());

        //通知房间里其他人
        self.room_member_notice(RoomMemberNoticeType::AddMember as u8, &user_id);
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
            if self.member_index.get(i).unwrap() != user_id {
                continue;
            }
            self.member_index.remove(i);
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

    pub fn get_next_choice_user(&self) -> u32 {
        let index = self.get_next_choice_index();
        self.get_choice_orders()[index]
    }

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
            //处理选择回合顺序
            self.remove_choice_order(chocie_index);
            if chocie_index < self.get_member_count() - 1 {
                self.set_next_choice_index(chocie_index + 1);
                self.build_choice_turn_task();
            }
        } else if self.state == RoomState::ChoiceIndex {
            //处理选择占位
            self.remove_choice_order(chocie_index);
            if chocie_index < self.get_member_count() - 1 {
                self.set_next_choice_index(chocie_index + 1);
                self.build_choice_index_task();
            }
        } else if self.state == RoomState::BattleStarted {
            //处理正在开始的战斗
            self.remove_turn_orders(turn_index);
            if self.get_next_turn_index() >= self.get_member_count() - 1 {
                self.set_next_turn_index(0);
            } else {
                self.set_next_turn_index(turn_index + 1);
            }
            self.build_battle_turn_task();
        }
    }

    ///换队伍
    pub fn change_team(&mut self, user_id: &u32, team_id: &u8) {
        let res = self.members.contains_key(user_id);
        if !res {
            return;
        }
        let mut sct = S_CHANGE_TEAM::new();
        sct.is_succ = true;
        self.send_2_client(
            ClientCode::ChangeTeam,
            *user_id,
            sct.write_to_bytes().unwrap(),
        );

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
        let member_size = self.member_index.len();
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
        self.battle_data.turn_orders = Vec::with_capacity(member_size);
    }

    pub fn random_location_order(&mut self) {}

    pub fn is_started(&self) -> bool {
        if self.state != RoomState::BattleStarted {
            false
        } else {
            true
        }
    }

    pub fn start(&mut self) {
        //生成地图
        self.tile_map = self.generate_map();
        //改变房间状态
        self.state = RoomState::ChoiceTurn;
        //下发通知
        self.start_notice();
    }

    pub fn get_choose_cters(&self) -> Vec<u32> {
        let mut cter_v = Vec::new();
        for member in self.members.values() {
            cter_v.push(member.chose_cter.cter_id);
        }
        cter_v
    }

    pub fn generate_map(&self) -> TileMap {
        let cter_v = self.get_choose_cters();
        let tmd = TileMap::init(&TEMPLATES, cter_v);
        info!("生成地图{:?}", tmd.clone());
        tmd
    }

    pub fn build_battle_turn_task(&self) {
        let next_turn_index = self.get_next_turn_index();
        let user_id = self.get_turn_orders()[next_turn_index];
        let time_limit = TEMPLATES
            .get_constant_ref()
            .temps
            .get("battle_turn_limit_time");
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
            warn!("the battle_turn_limit_time of Constant config is None!pls check!");
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

    pub fn build_choice_turn_task(&self) {
        let user_id = self.get_next_choice_user();
        //没选择完，继续选
        let time_limit = TEMPLATES.get_constant_ref().temps.get("choice_round_time");
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
            warn!("the choice_location_time of Constant config is None!pls check!");
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

    pub fn build_choice_index_task(&self) {
        let time_limit = TEMPLATES
            .get_constant_ref()
            .temps
            .get("choice_location_time");
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
            warn!("the choice_location_time of Constant config is None!pls check!");
        }
        task.cmd = TaskCmd::ChoiceLocation as u16;

        let mut map = serde_json::Map::new();
        map.insert(
            "user_id".to_owned(),
            serde_json::Value::from(self.get_next_choice_user()),
        );
        task.data = serde_json::Value::from(map);
        let res = self.task_sender.send(task);
        if res.is_err() {
            error!("{:?}", res.err().unwrap());
        }
    }
}
