use crate::entity::character::BattleCharacter;
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
    S_BATTLE_CHARACTER_NOTICE, S_CHANGE_TEAM, S_CHOOSE_LOCATION_NOTICE, S_CHOOSE_TURN_ORDER_NOTICE,
    S_EMOJI, S_EMOJI_NOTICE, S_KICK_MEMBER, S_PREPARE_CANCEL, S_ROOM, S_ROOM_MEMBER_LEAVE_NOTICE,
    S_ROOM_MEMBER_NOTICE, S_ROOM_NOTICE, S_START_NOTICE,
};
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

//最大成员数量
pub const MEMBER_MAX: u8 = 4;

pub enum RoomMemberNoticeType {
    AddMember = 1,
    UpdateMember = 2,
}

pub enum MemberLeaveNoticeType {
    Leave = 1,  //自己离开
    Kicked = 2, //被T
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
    id: u32,                              //房间id
    owner_id: u32,                        //房主id
    tile_map: TileMap,                    //地图数据
    pub members: HashMap<u32, Member>,    //玩家对应的队伍
    pub member_index: [u32; 4],           //玩家对应的位置
    pub turn_orders: [u32; 4],            //turn行动队列
    pub choice_orders: [u32; 4],          //选择顺序
    state: u8,                            //房间状态
    pub setting: RoomSetting,             //房间设置
    pub next_choice_user: u32,            //下一个选择占位玩家id
    pub next_turn_index: u32,             //下个turn玩家
    room_type: u8,                        //房间类型
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
        let member_index = [0; MEMBER_MAX as usize];
        let mut room = Room {
            id: id,
            owner_id: user_id,
            tile_map: TileMap::default(),
            members: HashMap::new(),
            member_index: member_index,
            turn_orders: [0; 4],
            choice_orders: [0; 4],
            state: RoomState::Await as u8,
            setting: RoomSetting::default(),
            next_choice_user: 0,
            next_turn_index: 0,
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
        room.member_index[0] = user_id;
        //返回客户端
        let mut sr = S_ROOM::new();
        sr.is_succ = true;
        sr.set_room(room.convert_to_pt());
        room.send_2_client(ClientCode::Room, user_id, sr.write_to_bytes().unwrap());
        Ok(room)
    }

    fn check_location(&mut self) -> bool {
        let mut user_id = 0;
        let mut res = true;
        for member_id in self.turn_orders.iter() {
            if *member_id == 0 {
                continue;
            }
            let member = self.members.get(member_id).unwrap();
            if member.chose_cter.location == 0 {
                user_id = member.user_id;
                res = false;
                break;
            }
        }
        if !res {
            self.next_choice_user = user_id;
            self.build_choice_location_task();
        }
        res
    }

    ///选择占位
    pub fn choice_location(&mut self, user_id: u32, index: Option<u32>) {
        let location;
        //玩家手动选的
        if let Some(index) = index {
            let member = self.members.get_mut(&user_id).unwrap();
            member.chose_cter.location = index;
            location = index;
        } else {
            //系统随机
            let mut random = rand::thread_rng();
            let v = self.tile_map.get_able_cells();
            let index = random.gen_range(0, v.len());
            let index = v.get(index).unwrap();
            let member = self.members.get_mut(&user_id).unwrap();
            member.chose_cter.location = *index;
            location = *index;
        }

        let mut scln = S_CHOOSE_LOCATION_NOTICE::new();
        scln.set_user_id(user_id);
        scln.location = location;
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

        //此处有两种情况
        //第一种，成员还没选择完占位，则继续定时器选择占位，否则进入选择回合顺序定时器

        let res = self.check_location();
        //都选择完了占位，进入选择回合顺序
        if res {
            self.build_choice_round_task();
        } else {
            //没选择完，继续选
            self.build_choice_location_task();
        }
    }

    ///选择回合
    pub fn choice_round(&mut self, user_id: u32, order: Option<u32>) {
        let turn_order;
        //如果玩家选择的
        if let Some(order) = order {
            self.turn_orders[order as usize] = user_id;
            turn_order = order;
        } else {
            //系统帮忙选
            let mut v = Vec::new();
            let mut index = 0;
            for i in self.choice_orders.iter() {
                if *i == 0 {
                    v.push(index);
                }
                index += 1;
            }
            let mut rand = rand::thread_rng();
            let res = rand.gen_range(0, v.len());
            let index = v.get(res).unwrap();
            turn_order = *index;
            self.turn_orders[turn_order as usize] = user_id;
        }

        //通知其他玩家
        let mut scron = S_CHOOSE_TURN_ORDER_NOTICE::new();
        scron.user_id = user_id;
        scron.order = turn_order;
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

        let res = self.check_choice_round();
        //如果都选完了，开始选占位，并发送战斗数据给客户端
        if res {
            let mut sbs = S_BATTLE_CHARACTER_NOTICE::new();
            self.cter_2_battle_cter();
            for member in self.members.values() {
                sbs.battle_cters.push(member.convert_to_battle_cter());
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

            //开始执行选择占位逻辑
            self.build_choice_location_task();

        //此处应该加上第一回合限制时间定时器
        } else {
            //如果没选完，继续选
            self.build_choice_round_task();
        }
    }

    pub fn check_choice_round(&mut self) -> bool {
        let mut user_id = 0;
        let mut res = true;
        for i in self.choice_orders.iter() {
            if *i == 0 {
                continue;
            }
            if !self.turn_orders.contains(i) {
                res = false;
                user_id = *i;
                break;
            }
        }
        if !res && user_id > 0 {
            self.next_choice_user = user_id;
        } else if res {
            self.next_choice_user = self.turn_orders[0];
        }
        res
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
        ssn.set_room_status(self.state as u32);
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
        let mut member_v = self.member_index.to_vec();
        let mut index = 0_u32;
        loop {
            if index >= (self.member_index.len() - 1) as u32 {
                break;
            }
            let rm_index = random.gen_range(0, member_v.len());
            let res = member_v.remove(rm_index);
            self.choice_orders[index as usize] = res;
            index += 1;
        }
        //此一次，所以直接取0下标的值
        self.next_choice_user = self.choice_orders[0];
        ssn.choice_order = self.choice_orders.to_vec();

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
        self.build_choice_round_task();
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

        //处理战斗相关的数据
        self.handler_leave(*user_id);
    }

    fn handler_leave(&mut self, user_id: u32) {
        let size = self.choice_orders.len();
        //如果下一个选择回合的玩家是离开的玩家，则选出下一个
        if self.next_choice_user == user_id {
            self.next_choice_user = 0;
            for i in 0..size {
                if self.choice_orders[i] == user_id {
                    let index = i + 1;
                    //在范围内，就选出下一个
                    if index <= size - 1 {
                        self.next_choice_user = self.choice_orders[index];
                        //选择回合定时器任务
                        if self.next_choice_user > 0 {
                            self.build_choice_round_task();
                        }
                    }
                }
            }
        }

        //如果下一次选择位置的玩家是离开的玩家就选出下一个
        if self.next_choice_user == user_id {
            self.next_choice_user = 0;
            for i in 0..size {
                if self.turn_orders[i] == user_id {
                    let index = i + 1;
                    //在范围内，就选出下一个
                    if index <= size - 1 {
                        self.next_choice_user = self.turn_orders[index];
                        //选择占位定时器任务
                        if self.next_choice_user > 0 {
                            self.build_choice_location_task();
                        }
                    }
                }
            }
        }

        //处理战斗相关的数据
        for i in 0..size {
            if self.choice_orders[i] == user_id {
                self.choice_orders[i] = 0;
            }
            if self.turn_orders[i] == user_id {
                self.turn_orders[i] = 0;
            }
        }

        //如果战斗已经开始了，下一个回合行动玩家是这个离线玩家，则轮到下一个玩家
        if self.turn_orders[self.next_turn_index as usize] == user_id {
            self.next_turn_index += 1;
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
        rp.set_room_status(self.state as u32);
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

    pub fn cter_2_battle_cter(&mut self) {
        for member in self.members.values_mut() {
            let battle_cter = BattleCharacter::init(&member.chose_cter);
            match battle_cter {
                Ok(b_cter) => {
                    member.battle_cter = b_cter;
                }
                Err(_) => {
                    return;
                }
            }
        }
    }

    pub fn random_location_order(&mut self) {}

    //是否已经开始了
    pub fn is_started(&self) -> bool {
        self.state == RoomState::Started as u8
    }

    pub fn start(&mut self) {
        //生成地图
        self.tile_map = self.generate_map();
        //改变房间状态
        self.state = RoomState::Started as u8;
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

    pub fn build_choice_round_task(&self) {
        let user_id = self.next_choice_user;
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
        task.cmd = TaskCmd::ChoiceRoundOrder as u16;

        let mut map = serde_json::Map::new();
        map.insert("user_id".to_owned(), serde_json::Value::from(user_id));
        task.data = serde_json::Value::from(map);
        let res = self.task_sender.send(task);
        if res.is_err() {
            error!("{:?}", res.err().unwrap());
        }
    }

    pub fn build_choice_location_task(&self) {
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
            serde_json::Value::from(self.next_choice_user),
        );
        task.data = serde_json::Value::from(map);
        let res = self.task_sender.send(task);
        if res.is_err() {
            error!("{:?}", res.err().unwrap());
        }
    }
}
