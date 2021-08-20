use crate::battle::battle_cter::BattleCharacter;
use crate::battle::battle_trigger::TriggerEvent;
use crate::battle::mission::random_mission;
use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::map_data::TileMap;
use crate::room::member::Member;
use crate::room::{RoomSetting, RoomState, RoomType, MEMBER_MAX};
use crate::task_timer::{Task, TaskCmd};
use crate::{JsonValue, TEMPLATES};
use chrono::{DateTime, Utc};
use crossbeam::channel::Sender;
use log::{error, info, warn};
use protobuf::Message;
use rand::Rng;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;
use tools::cmd_code::{ClientCode, GameCode, RoomCode};
use tools::macros::GetMutRef;
use tools::protos::base::{RoomPt, WorldCellPt};
use tools::protos::battle::{
    S_BATTLE_START_NOTICE, S_CHOOSE_INDEX_NOTICE, S_MAP_REFRESH_NOTICE, S_START_NOTICE,
};
use tools::protos::room::{S_EMOJI, S_EMOJI_NOTICE, S_ROOM_MEMBER_LEAVE_NOTICE};
use tools::protos::server_protocol::{B_R_G_PUNISH_MATCH, B_R_SUMMARY};
use tools::util::packet::Packet;

use super::MemberLeaveNoticeType;

///房间结构体，封装房间必要信息
#[derive(Clone)]
pub struct Room {
    id: u32,                             //房间id
    room_type: RoomType,                 //房间类型
    owner_id: u32,                       //房主id
    pub state: RoomState,                //房间状态
    pub members: HashMap<u32, Member>,   //玩家id对应角色id
    pub member_index: [u32; MEMBER_MAX], //玩家对应的位置
    pub setting: RoomSetting,            //房间设置
    pub battle_data: BattleData,         //战斗相关数据封装
    pub tcp_sender: Sender<Vec<u8>>,     //tcpsender
    task_sender: Sender<Task>,           //任务sender
    robot_sender: Sender<RobotTask>,     //机器人sender
    time: DateTime<Utc>,                 //房间创建时间
}

tools::get_mut_ref!(Room);

impl Room {
    ///构建一个房间的结构体
    pub fn new(
        rp: &RoomPt,
        tcp_sender: Sender<Vec<u8>>,
        task_sender: Sender<Task>,
        robot_sender: Sender<RobotTask>,
    ) -> anyhow::Result<Room> {
        let owner_id = rp.owner_id;
        let room_id = rp.room_id;
        let room_type = RoomType::try_from(rp.room_type as u8);
        if let Err(e) = room_type {
            anyhow::bail!("{:?}", e)
        }
        let room_type = room_type.unwrap();
        let mut members = HashMap::new();
        let mut member_index: [u32; MEMBER_MAX] = [0; MEMBER_MAX];
        let mut index = 0;
        for member_pt in rp.members.iter() {
            members.insert(member_pt.user_id, Member::from(member_pt));
            member_index[index] = member_pt.user_id;
            index += 1;
        }
        let room_setting = RoomSetting::from(rp.setting.as_ref().unwrap());

        //转换成tilemap数据
        let time = Utc::now();
        let room = Room {
            id: room_id,
            owner_id,
            members,
            member_index,
            state: RoomState::ChoiceIndex,
            setting: room_setting,
            battle_data: BattleData::new(room_type, task_sender.clone(), tcp_sender.clone()),
            room_type,
            tcp_sender,
            task_sender,
            robot_sender,
            time,
        };
        Ok(room)
    }

    pub fn get_last_order_user(&self) -> u32 {
        let mut index = (MEMBER_MAX - 1) as usize;
        loop {
            let res = self.battle_data.turn_orders[index];
            if res != 0 {
                return res;
            }
            if index == 0 {
                return res;
            }
            index -= 1;
        }
    }

    pub fn add_punish(&mut self, user_id: u32) {
        let res = self.members.get_mut(&user_id);
        if res.is_none() {
            return;
        }
        let member = res.unwrap();
        member.punish_match.add_punish();
        let mut brg = B_R_G_PUNISH_MATCH::new();
        brg.set_punish_match(member.punish_match.into());
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

    //随便获得一个玩家,如果玩家id==0,则代表没有玩家了
    pub fn random_user_id(&self) -> u32 {
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

    //离开房间结算
    pub fn league_summary(&mut self, user_id: u32) {
        let mut need_summary = false;
        let mut punishment = false;
        let room_state = self.state;
        let res = self.battle_data.get_battle_player_mut(Some(user_id), false);
        if let Err(_) = res {
            return;
        }
        let battle_player = res.unwrap();

        //房间必须为选择占位阶段和开始战斗阶段
        if room_state == RoomState::ChoiceIndex || room_state == RoomState::BattleStarted {
            need_summary = true;

            //没死走惩罚逻辑，死了走正常逻辑
            if !battle_player.is_died() {
                punishment = true;
            }
            //离开房间，当死亡处理
            self.battle_data.leave_user = (user_id, punishment);
        }
        //走惩罚触发
        if need_summary && punishment {
            let str = format!("player die!because leave and summary!user_id:{}", user_id);
            self.battle_data.after_player_died_trigger(
                user_id,
                user_id,
                true,
                punishment,
                Some(str),
            );
        }
    }

    ///处理战斗结算,外部需要判断这个玩家在不在房间里
    ///返回是否结算，是否刷新地图
    pub unsafe fn battle_summary(&mut self) -> bool {
        if self.state != RoomState::ChoiceIndex && self.state != RoomState::BattleStarted {
            return false;
        }
        let is_battle_over;
        let summary_protos = self.battle_data.summary();
        //发给游戏服同步结算数据
        if !summary_protos.is_empty() && self.room_type.is_match_type() {
            for sp in summary_protos {
                let user_id = sp.get_summary_data().user_id;
                let res = sp.write_to_bytes();
                match res {
                    Ok(bytes) => {
                        self.send_2_server(GameCode::Summary.into_u32(), user_id, bytes);
                    }
                    Err(e) => {
                        error!("{:?}", e)
                    }
                }
            }
        }

        let size = self.battle_data.get_alive_player_num();
        if size <= 1 {
            is_battle_over = true;
            self.state = RoomState::BattleOvered;
            self.battle_data.state = RoomState::BattleOvered;
        } else {
            is_battle_over = false;
        }

        //发给房间服
        if self.state == RoomState::BattleOvered {
            let user_id = self.random_user_id();
            if user_id > 0 {
                let mut proto = B_R_SUMMARY::new();
                proto.room_type = self.room_type.into_u32();
                proto.room_id = self.get_room_id();
                let bytes = proto.write_to_bytes();
                match bytes {
                    Ok(bytes) => {
                        //通知房间服战斗结算
                        self.send_2_server(RoomCode::Summary.into_u32(), 0, bytes);
                    }
                    Err(err) => {
                        error!("{:?}", err);
                    }
                }
            }
        }
        is_battle_over
    }

    ///刷新地图
    pub fn refresh_map(&mut self) -> bool {
        let need_refresh = self.battle_data.check_refresh_map();
        if !need_refresh {
            return false;
        }

        let season_is_open = self.setting.season_is_open;
        let last_map_id = self.battle_data.last_map_id;
        let res = self.battle_data.reset_map(season_is_open, last_map_id);
        if let Err(e) = res {
            error!("{:?}", e);
            return false;
        }
        let mut smrn = S_MAP_REFRESH_NOTICE::new();
        smrn.room_status = self.state as u32;
        smrn.tile_map_id = self.battle_data.tile_map.id;
        if self.battle_data.tile_map.world_cell.1 > 0 {
            let mut wcp = WorldCellPt::new();
            wcp.index = self.battle_data.tile_map.world_cell.0 as u32;
            wcp.world_cell_id = self.battle_data.tile_map.world_cell.1;
            smrn.world_cell.push(wcp);
        }
        for battle_player in self.battle_data.battle_player.values() {
            smrn.battle_players
                .push(battle_player.convert_to_battle_player_pt());
        }
        let bytes = smrn.write_to_bytes().unwrap();
        let self_mut_ref = self.get_mut_ref();
        for &id in self.member_index.iter() {
            self_mut_ref.send_2_client(ClientCode::MapRefreshNotice, id, bytes.clone());
        }
        self.start_choice_index();
        true
    }

    ///回其他服消息
    pub fn send_2_server(&mut self, cmd: u32, user_id: u32, bytes: Vec<u8>) {
        let battle_player = self.get_battle_player_ref(&user_id);
        match battle_player {
            Some(battle_player) => {
                if battle_player.is_robot() {
                    return;
                }
            }
            _ => {}
        }
        let bytes = Packet::build_packet_bytes(cmd, user_id, bytes, true, false);
        let res = self.tcp_sender.send(bytes);
        if let Err(e) = res {
            error!("{:?}", e);
        }
    }

    ///开始选择占位
    ///修改房间状态为RoomState::ChoiceIndex
    ///将角色转换成战斗角色
    ///创建选择下标检测任务
    pub fn start_choice_index(&mut self) {
        self.state = RoomState::ChoiceIndex;
        self.battle_data.state = RoomState::ChoiceIndex;
        info!(
            "change room state RoomState::ChoiceIndex,because start_choice_index!room_id:{}",
            self.id
        );
        //地图没有刷新过就把cter转换成battle_cter
        if self.battle_data.reflash_map_turn.is_none() {
            self.cter_2_battle_cter();
        }
        info!(
            "start_choice_index!turn_order:{:?}",
            self.battle_data.turn_orders
        );
        //增加round
        self.battle_data.round += 1;
        //开始执行占位逻辑
        self.build_choice_index_task();
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

    ///推进下一个人，并检测状态，如果都选择完了展位，切换到战斗已经开始状态
    pub fn check_next_choice_index(&mut self) {
        let world_boss_init_index = self.battle_data.tile_map.world_boss_init_index;

        let mut res = true;

        //判断是否都选完了
        for battle_player in self.battle_data.battle_player.values() {
            if battle_player.is_died() {
                continue;
            }
            if !battle_player.get_major_cter().map_cell_index_is_choiced() {
                res = false;
                break;
            }
        }
        if res {
            self.state = RoomState::BattleStarted;
            self.battle_data.state = RoomState::BattleStarted;
            info!("change room state RoomState::BattleStarted,because check_next_choice_index and all done!room_id:{}",self.id);
        } else {
            self.battle_data.choice_index_next_turn();

            //帮worldboss选
            let player = self.battle_data.get_battle_player_mut(None, false);
            if let Ok(player) = player {
                if player.is_world_boss {
                    let cter_id = player.get_major_cter_mut().get_cter_id();
                    player
                        .get_major_cter_mut()
                        .set_map_cell_index(world_boss_init_index);
                    let map_cell = self
                        .battle_data
                        .tile_map
                        .map_cells
                        .get_mut(world_boss_init_index)
                        .unwrap();
                    map_cell.cter_id = cter_id;
                    self.check_next_choice_index();
                    return;
                }
            }
        }

        if self.state == RoomState::BattleStarted {
            self.battle_data.next_turn(false);
        }
    }

    pub fn is_all_robot(&self) -> bool {
        for member in self.members.values() {
            if member.robot_temp_id == 0 {
                return false;
            }
        }
        true
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

    pub fn get_next_turn_index(&self) -> usize {
        self.battle_data.next_turn_index
    }

    pub fn set_next_turn_index(&mut self, index: usize) {
        self.battle_data.next_turn_index = index;
    }

    pub fn get_battle_player_ref(&self, key: &u32) -> Option<&BattlePlayer> {
        self.battle_data.battle_player.get(key)
    }

    pub fn get_battle_player_mut_ref(&mut self, key: &u32) -> Option<&mut BattlePlayer> {
        self.battle_data.battle_player.get_mut(key)
    }

    pub fn get_battle_cter_ref(&self, key: &u32) -> Option<&BattleCharacter> {
        let res = self.battle_data.battle_player.get(key);
        if res.is_none() {
            return None;
        }
        let battle_player = res.unwrap();
        battle_player.cters.get(key)
    }

    pub fn get_battle_cter_mut(&mut self, cter_id: u32) -> anyhow::Result<&mut BattleCharacter> {
        self.battle_data.get_battle_cter_mut(cter_id, true)
    }

    pub fn check_index_over(&mut self) -> bool {
        self.state == RoomState::BattleStarted
    }

    ///选择占位
    pub fn choice_index(&mut self, cter_id: u32, index: u32) {
        let user_id = self.battle_data.get_user_id(cter_id).unwrap();
        let turn_index = self.get_next_turn_index();
        let turn_order = self.battle_data.turn_orders;
        let battle_cter = self.get_battle_cter_mut(cter_id);
        if let Err(e) = battle_cter {
            error!("{:?}", e);
            return;
        }
        let battle_cter = battle_cter.unwrap();

        info!(
            "choice choice_index battle_cter:{},index:{},turn_order:{:?}",
            cter_id, turn_index, turn_order
        );

        //更新角色下标和地图块上面的角色id
        battle_cter.set_map_cell_index(index as usize);
        let map_cell = self
            .battle_data
            .tile_map
            .map_cells
            .get_mut(index as usize)
            .unwrap();
        map_cell.cter_id = cter_id;
        let mut scln = S_CHOOSE_INDEX_NOTICE::new();
        scln.set_cter_id(cter_id);
        scln.set_user_id(user_id);
        scln.set_index(index);
        let bytes = scln.write_to_bytes().unwrap();
        let self_mut_ref = self.get_mut_ref();
        //通知给房间成员
        for id in self.battle_data.battle_player.keys() {
            self_mut_ref.send_2_client(ClientCode::ChoiceIndexNotice, *id, bytes.clone());
        }

        //添加下个turnindex
        self.check_next_choice_index();

        //下一个人初始化任务
        self.init_mission_for_choice_index();

        //选完了就进入战斗
        let res = self.check_index_over();
        //都选择完了占位，进入选择回合顺序
        if res {
            self.push_start_battle();
            let battle_data_ptr = &mut self.battle_data as *mut BattleData;
            let battle_player = self.battle_data.get_battle_player_mut(None, false).unwrap();
            let current_cter_is_died = battle_player.is_died();
            //如果角色没死，并且是机器人，则通知机器人执行完了,并且启动机器人action
            if !current_cter_is_died || battle_player.robot_data.is_some() {
                battle_player.robot_start_action(battle_data_ptr);
            }
        } else {
            //没选择完，继续选
            self.build_choice_index_task();
        }
    }

    //给没选都人随机回合顺序
    pub fn init_turn_order(&mut self) {
        //初始化段位快照
        self.init_league_map();
        //初始化战斗角色
        self.cter_2_battle_cter();
        //先选出可以随机的下标
        let member_count = self.members.len();
        let mut index_v: Vec<usize> = Vec::new();
        for index in 0..member_count {
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

        //检测出场顺位，没有选的，系统进行随机
        let room = self as *mut Room;
        unsafe {
            //设置worldboss行动顺序
            if self.room_type.is_boss_type() {
                let world_boss = self.battle_data.get_world_boss_ref().unwrap();
                let cter_temp_id = world_boss.get_major_cter().get_cter_temp_id();
                let world_boss_temp = crate::TEMPLATES
                    .worldboss_temp_mgr()
                    .temps
                    .get(&cter_temp_id)
                    .unwrap();
                let index = world_boss_temp.turn_order_index;
                let res;
                match index {
                    0 => {
                        res = rand.gen_range(0..index_v.len());
                    }
                    1 => {
                        res = rand.gen_range(0..1);
                    }
                    2 => {
                        res = rand.gen_range(index_v.len() - 1..index_v.len());
                    }
                    _ => {
                        res = rand.gen_range(0..index_v.len());
                    }
                }
                let res_index = index_v.remove(res);
                let id = world_boss.user_id;
                self.insert_turn_orders(res_index, id);
            }
            for member_id in room.as_ref().unwrap().members.keys() {
                let member_id = *member_id;
                //选过了就跳过
                if self.turn_order_contains(&member_id) {
                    continue;
                }
                //系统帮忙选
                let remove_index = rand.gen_range(0..index_v.len());
                let index = index_v.get(remove_index).unwrap();
                let turn_order = *index as usize;
                self.insert_turn_orders(turn_order, member_id);
                index_v.remove(remove_index);
            }
        }
        //行动turn索引从0开始
        self.set_next_turn_index(0);
        //如果第一个玩家是0,则跳过到下一个,主要是避免刷新地图的时候出bug
        let next_turn_user = self.get_turn_user(None).unwrap();
        if next_turn_user == 0 {
            self.check_next_choice_index();
        }
        //开始选择下标
        self.start_choice_index();
    }

    pub fn turn_order_contains(&self, user_id: &u32) -> bool {
        self.battle_data.turn_orders.contains(user_id)
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let member = self.battle_data.battle_player.get(&user_id);
        if let None = member {
            return;
        }
        let member = member.unwrap();
        //如果是机器人，则返回，不发送
        if member.is_robot() {
            return;
        }
        let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes, true, true);
        let res = self.tcp_sender.send(bytes);
        if let Err(e) = res {
            error!("{:?}", e);
        }
    }

    pub fn send_2_all_client(&mut self, cmd: ClientCode, bytes: Vec<u8>) {
        let mut user_id;
        for member in self.members.values_mut() {
            user_id = member.user_id;
            let battle_player = self.battle_data.battle_player.get(&user_id);
            //如果是机器人，则返回，不发送
            if battle_player.is_none() {
                continue;
            }
            let battle_player = battle_player.unwrap();
            if battle_player.is_robot() {
                continue;
            }
            let bytes = Packet::build_packet_bytes(cmd as u32, user_id, bytes.clone(), true, true);
            let res = self.tcp_sender.send(bytes);
            if let Err(e) = res {
                error!("{:?}", e);
            }
        }
    }

    pub fn init_mission_for_choice_index(&mut self) {
        let battle_player = self.battle_data.get_battle_player_mut(None, false);
        if let Err(e) = battle_player {
            error!("{:?}", e);
            return;
        }
        let battle_player = battle_player.unwrap();
        if battle_player.mission_data.mission.is_some() || battle_player.is_world_boss {
            return;
        }
        let user_id = battle_player.get_user_id();
        random_mission(self.battle_data.borrow_mut(), user_id);
    }

    //战斗通知
    pub fn start_notice(&mut self) {
        let mut ssn = S_START_NOTICE::new();
        ssn.set_room_status(self.state as u32);
        ssn.set_tile_map_id(self.battle_data.tile_map.id);
        //封装世界块
        if self.battle_data.tile_map.world_cell.1 > 0 {
            let mut wcp = WorldCellPt::default();
            wcp.set_index(self.battle_data.tile_map.world_cell.0 as u32);
            wcp.set_world_cell_id(self.battle_data.tile_map.world_cell.1);
            ssn.world_cell.push(wcp);
        }
        //封装turn order
        for index in self.battle_data.turn_orders.iter() {
            ssn.turn_order.push(*index);
        }
        for battle_player in self.battle_data.battle_player.values() {
            let mut battle_player_pt = battle_player.convert_to_battle_player_pt();
            if battle_player.is_world_boss {
                for bp in battle_player_pt.cters.iter_mut() {
                    if bp.cter_id == battle_player.major_cter.0 {
                        bp.set_index(self.battle_data.tile_map.world_boss_init_index as u32);
                    }
                }
            }
            ssn.battle_players.push(battle_player_pt);
        }
        let bytes = ssn.write_to_bytes().unwrap();
        let self_mut_ref = self.get_mut_ref();
        for id in self.members.keys() {
            self_mut_ref.send_2_client(ClientCode::StartNotice, *id, bytes.clone());
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
    pub fn member_leave_notice(
        &mut self,
        notice_type: MemberLeaveNoticeType,
        user_id: &u32,
        need_push_self: bool,
    ) {
        let mut srmln = S_ROOM_MEMBER_LEAVE_NOTICE::new();
        srmln.set_notice_type(notice_type.into_u32());
        srmln.set_user_id(*user_id);
        let bytes = srmln.write_to_bytes().unwrap();
        let self_mut_ref = self.get_mut_ref();
        for member_id in self.members.keys() {
            if !need_push_self && member_id == user_id {
                continue;
            }
            self_mut_ref.send_2_client(ClientCode::MemberLeaveNotice, *member_id, bytes.clone());
        }

        if notice_type == MemberLeaveNoticeType::Kicked {
            self.send_2_server(RoomCode::BattleKickMember.into_u32(), *user_id, vec![]);
        }
    }

    pub fn get_state(&self) -> RoomState {
        self.state
    }

    ///获得房间类型
    pub fn get_room_type(&self) -> RoomType {
        self.room_type
    }

    ///获取房号
    pub fn get_room_id(&self) -> u32 {
        self.id
    }

    pub fn handler_punish(&mut self, user_id: u32) {
        if !self.room_type.is_match_type() {
            return;
        }
        let battle_player = self.get_battle_player_mut_ref(&user_id).unwrap();
        if battle_player.is_died() {
            return;
        }
        //self.add_punish(user_id);
    }

    ///移除玩家
    pub fn remove_member(
        &mut self,
        notice_type: MemberLeaveNoticeType,
        user_id: &u32,
        need_push_self: bool,
    ) {
        let res = self.members.get_mut(user_id);
        if res.is_none() {
            return;
        }

        //处理匹配惩罚
        self.handler_punish(*user_id);

        //通知客户端
        self.member_leave_notice(notice_type, user_id, need_push_self);

        //处理战斗相关的数据
        self.handler_leave(*user_id);
    }

    fn get_turn_orders(&self) -> &[u32] {
        &self.battle_data.turn_orders[..]
    }

    pub fn get_turn_user(&self, index: Option<usize>) -> anyhow::Result<u32> {
        self.battle_data.get_turn_user(index)
    }

    ///处理选择占位时候的离开
    fn handler_leave_choice_index(&mut self, user_id: u32, turn_index: usize) {
        let next_turn_user = self.get_turn_user(None);
        if let Err(e) = next_turn_user {
            error!("{:?}", e);
            return;
        }
        let next_turn_user = next_turn_user.unwrap();

        let last_order_user = self.get_last_order_user();
        self.remove_turn_orders(turn_index);

        //移除战斗角色
        self.battle_data.remove_player(user_id);
        //如果当前离开的玩家不是当前顺序就退出
        if next_turn_user != user_id {
            return;
        }
        //如果当前玩家正好处于最后一个顺序
        if last_order_user == user_id {
            self.state = RoomState::BattleStarted;
            self.battle_data.state = RoomState::BattleStarted;
            info!("change room state RoomState::BattleStarted,because handler_leave_choice_index last_order_user is:{},room_id:{}",last_order_user,self.id);
            self.set_next_turn_index(0);
            let next_turn_user = self.get_turn_user(None).unwrap();
            if next_turn_user == 0 {
                self.check_next_choice_index();
            }
            self.push_start_battle();
        } else {
            //不是最后一个就轮到下一个
            self.check_next_choice_index();
            //初始化玩家任务
            self.init_mission_for_choice_index();
            self.build_choice_index_task();
        }
    }

    ///处理选择战斗回合时候的离开
    fn handler_leave_battle_turn(&mut self, user_id: u32, turn_index: usize) {
        let next_turn_user = self.get_turn_user(None);
        if let Err(e) = next_turn_user {
            error!("{:?}", e);
            return;
        }
        let next_turn_user = next_turn_user.unwrap();
        //移除顺位数据
        self.remove_turn_orders(turn_index);
        //移除玩家战斗数据
        self.battle_data.remove_player(user_id);
        //如果当前离开的玩家不是当前顺序就退出
        if next_turn_user != user_id {
            return;
        }
        self.battle_data.next_turn(true);
    }

    ///处理玩家离开
    fn handler_leave(&mut self, user_id: u32) {
        let mut turn_index = 0;

        //处理段位结算
        self.league_summary(user_id);

        //处理战斗结算
        unsafe {
            self.battle_summary();
        }

        //找出离开玩家的回合下标
        for i in self.get_turn_orders() {
            if i == &user_id {
                break;
            }
            turn_index += 1;
        }

        //删除各项数据
        match self.state {
            RoomState::ChoiceIndex => {
                self.handler_leave_choice_index(user_id, turn_index);
            }
            RoomState::BattleStarted => {
                self.handler_leave_battle_turn(user_id, turn_index);
            }
            _ => {}
        }

        //删除数据
        self.members.remove(&user_id);
        //删除玩家数组的下标
        for i in 0..self.member_index.len() {
            if self.member_index[i] != user_id {
                continue;
            }
            self.member_index[i] = 0;
            break;
        }
    }

    ///判断房间是否有成员
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    pub fn cter_2_battle_cter(&mut self) {
        let ai_level = self.setting.ai_level as u32;
        for member in self.members.values_mut() {
            let battle_player = BattlePlayer::init(
                &member,
                &mut self.battle_data,
                ai_level,
                self.robot_sender.clone(),
            );
            match battle_player {
                Ok(battle_player) => {
                    let cter_id = battle_player.current_cter.0;
                    let user_id = member.user_id;
                    self.battle_data
                        .battle_player
                        .insert(member.user_id, battle_player);
                    self.battle_data.cter_player.insert(cter_id, user_id);
                }
                Err(_) => {
                    return;
                }
            }
        }
    }

    ///开始游戏
    pub fn start(&mut self) {
        //初始化turn顺序
        self.init_turn_order();
        //生成地图
        let res = self.generate_map();
        if let Err(e) = res {
            error!("{:?}", e);
            return;
        }
        //下发通知
        self.start_notice();
        //初始化玩家任务
        self.init_mission_for_choice_index();
    }

    pub fn init_league_map(&mut self) {
        for member in self.members.values() {
            let user_id = member.user_id;
            let league_id = member.league.get_league_id();
            self.battle_data.leave_map.insert(user_id, league_id);
        }
    }

    ///生成地图
    pub fn generate_map(&mut self) -> anyhow::Result<()> {
        let season_is_open = self.setting.season_is_open;
        let last_map_id = self.battle_data.last_map_id;
        let tmd = TileMap::init(&mut self.battle_data, season_is_open, last_map_id)?;

        self.battle_data.last_map_id = tmd.id;
        self.battle_data.tile_map = tmd;
        self.battle_data.turn_limit_time = self.setting.turn_limit_time as u64;
        Ok(())
    }

    ///战斗开始
    pub fn push_start_battle(&mut self) {
        //判断是否有世界块,有的话，
        if self.battle_data.tile_map.world_cell.1 > 0 && self.battle_data.reflash_map_turn.is_none()
        {
            let world_cell_id = self.battle_data.tile_map.world_cell.1;
            let world_cell_temp = TEMPLATES
                .world_cell_temp_mgr()
                .temps
                .get(&world_cell_id)
                .unwrap();

            for buff_id in world_cell_temp.buff.iter() {
                let buff = TEMPLATES.buff_temp_mgr().get_temp(&buff_id);
                if let Err(e) = buff {
                    error!("{:?}", e);
                    continue;
                }
                let buff = buff.unwrap();
                if buff.par1 > 0 {
                    for (_, battle_player) in self.battle_data.battle_player.iter_mut() {
                        battle_player.get_current_cter_mut().add_buff(
                            None,
                            None,
                            buff.par1,
                            Some(self.battle_data.next_turn_index),
                        );
                    }
                }
            }
        }
        let mut sbsn = S_BATTLE_START_NOTICE::new();
        let debug = crate::CONF_MAP.borrow().get_bool("debug");
        for battle_player in self.battle_data.battle_player.values() {
            let battle_player_pt = battle_player.convert_to_battle_player_pt();
            sbsn.battle_players.push(battle_player_pt);
        }
        if debug {
            sbsn.map_data = self.battle_data.tile_map.to_json_for_debug();
            println!("{:?}", sbsn.map_data);
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
        if self.setting.turn_limit_time == 0 {
            return;
        }
        let user_id = self.get_turn_user(None);
        if let Err(e) = user_id {
            error!("{:?}", e);
            return;
        }
        let user_id = user_id.unwrap();
        let time_limit = TEMPLATES.constant_temp_mgr().temps.get("choice_index_time");
        let mut task = Task::default();

        if self.room_type.is_custom_type() {
            task.delay = self.setting.turn_limit_time as u64;
        } else {
            if let Some(time) = time_limit {
                let time = u64::from_str(time.value.as_str());
                match time {
                    Ok(time) => {
                        task.delay = time + 500;
                    }
                    Err(e) => {
                        task.delay = 20000_u64;
                        error!("{:?}", e);
                    }
                }
            } else {
                task.delay = 5000_u64;
                warn!("the choice_index_time of Constant config is None!pls check!");
            }
        }

        task.cmd = TaskCmd::ChoiceIndex;

        let mut map = serde_json::Map::new();
        map.insert("user_id".to_owned(), JsonValue::from(user_id));
        task.data = JsonValue::from(map);
        let res = self.task_sender.send(task);
        if res.is_err() {
            error!("{:?}", res.err().unwrap());
        }
    }
}
