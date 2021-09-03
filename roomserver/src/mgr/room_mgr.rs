use crate::handlers::room_handler::{
    battle_kick_member, cancel_search_room, change_team, choice_ai, choice_skills,
    choose_character, confirm_into_room, create_room, emoji, join_room, kick_member, leave_room,
    off_line, prepare_cancel, reload_temps, room_setting, search_room, start, summary,
    update_season, update_worldboss,
};
use crate::room::custom_room::CustomRoom;
use crate::room::match_room::MatchRoom;
use crate::room::room::{Room, RoomState};
use crate::room::room_model::{RoomModel, RoomType};
use crate::room::world_boss_custom_room::WorldBossCustomRoom;
use crate::room::world_boss_match_room::WorldBossMatchRoom;
use crate::task_timer::Task;
use crossbeam::channel::Sender;
use log::{error, info, warn};
use rayon::slice::ParallelSliceMut;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::convert::TryFrom;
use tools::cmd_code::{ClientCode, RoomCode, ServerCommonCode};
use tools::tcp_message_io::TcpHandler;
use tools::util::packet::Packet;

type CmdFn = HashMap<u32, fn(&mut RoomMgr, Packet), RandomState>;

///房间服管理器
#[derive(Default)]
pub struct RoomMgr {
    pub custom_room: CustomRoom,                     //自定义房
    pub match_room: MatchRoom,                       //公共房
    pub world_boss_match_room: WorldBossMatchRoom,   //世界boss自定义房间
    pub world_boss_custom_room: WorldBossCustomRoom, //世界boss自定义房间
    pub player_room: HashMap<u32, u64>, //玩家对应的房间，key:u32,value:采用一个u64存，通过位运算分出高低位,低32位是房间模式,高32位是房间id
    pub cmd_map: CmdFn,                 //命令管理 key:cmd,value:函数指针
    tcp_handler: Option<TcpHandler>,    //tcp channel的发送方
    pub task_sender: Option<Sender<Task>>, //task channel的发送方
}

tools::get_mut_ref!(RoomMgr);

impl RoomMgr {
    pub fn new() -> RoomMgr {
        let mut rm = RoomMgr::default();
        rm.cmd_init();
        rm
    }

    pub fn get_room_mut(&mut self, room_type: RoomType, room_id: u32) -> anyhow::Result<&mut Room> {
        let res = match room_type {
            RoomType::OneVOneVOneVOneCustom => self.custom_room.get_room_mut(&room_id),
            RoomType::OneVOneVOneVOneMatch => self.match_room.get_room_mut(&room_id),
            RoomType::WorldBossCustom => self.world_boss_custom_room.get_room_mut(&room_id),
            RoomType::WorldBoseMatch => self.world_boss_match_room.get_room_mut(&room_id),
            _ => None,
        };
        if res.is_none() {
            anyhow::bail!(
                "there is no Room! room_type:{:?},room_id:{}",
                room_type,
                room_id
            )
        }
        Ok(res.unwrap())
    }

    pub fn rm_room_without_push(&mut self, room_type: RoomType, room_id: u32) {
        let room = match room_type {
            RoomType::OneVOneVOneVOneCustom => self.custom_room.rm_room(&room_id),
            RoomType::OneVOneVOneVOneMatch => self.match_room.rm_room(&room_id),
            RoomType::WorldBossCustom => self.world_boss_custom_room.rm_room(&room_id),
            RoomType::WorldBoseMatch => self.world_boss_match_room.rm_room(&room_id),
            _ => None,
        };
        if room.is_none() {
            return;
        }
        let room = room.unwrap();
        if room.is_empty() {
            return;
        }
        for member in room.members.keys() {
            self.player_room.remove(member);
        }
    }

    ///删除房间成员，不推送给客户端
    ///user_id:玩家id
    pub fn remove_member_without_push(&mut self, user_id: u32) {
        let res = self.player_room.get(&user_id);
        if res.is_none() {
            return;
        }
        let res = res.unwrap();
        let (room_type, room_id) = tools::binary::separate_long_2_int(*res);
        let room_type = RoomType::try_from(room_type as u8);
        if let Err(e) = room_type {
            error!("{:?}", e);
            return;
        }
        let room_type = room_type.unwrap();
        let room;
        room = match room_type {
            RoomType::OneVOneVOneVOneCustom => self.custom_room.get_room_mut(&room_id),
            RoomType::OneVOneVOneVOneMatch => self.match_room.get_room_mut(&room_id),
            RoomType::WorldBossCustom => self.world_boss_custom_room.get_room_mut(&room_id),
            RoomType::WorldBoseMatch => self.world_boss_match_room.get_room_mut(&room_id),
            _ => None,
        };

        if let None = room {
            warn!("the room is None!user_id:{}", user_id);
            return;
        }
        let room = room.unwrap();
        let room_state = room.state;
        let room_id = room.get_room_id();
        let room_type = room.get_room_type();
        room.remove_member_without_push(user_id);
        room.robots.remove(&user_id);
        self.player_room.remove(&user_id);
        let need_rm_room = room.check_need_rm_room();
        info!(
            "玩家退出房间!删除房间内玩家数据!不通知客户端!user_id:{},room_id:{}",
            user_id, room_id
        );

        if room_type.is_match_type() && room_state == RoomState::AwaitConfirm {
            let mut need_rm_cache = false;
            let mut need_cache_sort = false;

            let room_cache_iter = if room_type == RoomType::OneVOneVOneVOneMatch {
                self.match_room.room_cache.iter_mut()
            } else {
                self.world_boss_match_room.room_cache.iter_mut()
            };

            for room_cache in room_cache_iter {
                if room_cache.room_id != room_id {
                    continue;
                }
                if room_cache.count > 0 {
                    room_cache.count -= 1;
                }
                if room_cache.count == 0 {
                    need_rm_cache = true;
                }
                need_cache_sort = true;

                break;
            }
            if !need_rm_room && need_rm_cache {
                self.remove_room_cache(room_type, room_id);
            }
            if !need_rm_room && need_cache_sort {
                //重新排序
                self.sort_for_match_room(room_type);
            }
        }
        if need_rm_room {
            self.rm_room_without_push(room_type, room_id);
        }
    }

    pub fn sort_for_match_room(&mut self, room_type: RoomType) {
        if room_type == RoomType::OneVOneVOneVOneMatch {
            self.match_room
                .room_cache
                .par_sort_by(|a, b| b.count.cmp(&a.count));
        } else if room_type == RoomType::WorldBoseMatch {
            self.world_boss_match_room
                .room_cache
                .par_sort_by(|a, b| b.count.cmp(&a.count));
        }
    }

    pub fn remove_room_cache(&mut self, room_type: RoomType, room_id: u32) {
        if room_type == RoomType::OneVOneVOneVOneMatch {
            self.match_room.remove_room_cache(&room_id);
        } else if room_type == RoomType::WorldBoseMatch {
            self.world_boss_match_room.remove_room_cache(&room_id);
        }
    }

    pub fn get_task_sender_clone(&self) -> crossbeam::channel::Sender<Task> {
        self.task_sender.as_ref().unwrap().clone()
    }

    pub fn send_2_client(&mut self, cmd: ClientCode, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd.into_u32(), user_id, bytes, true, true);
        let tcp = self.tcp_handler.as_ref().unwrap();
        let endpoint = tcp.endpoint;
        tcp.node_handler.network().send(endpoint, bytes.as_slice());
    }

    pub fn send_2_server(&mut self, cmd: u32, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd, user_id, bytes, true, false);
        let tcp = self.tcp_handler.as_ref().unwrap();
        let endpoint = tcp.endpoint;
        tcp.node_handler.network().send(endpoint, bytes.as_slice());
    }

    pub fn set_tcp_handler(&mut self, sender: TcpHandler) {
        self.tcp_handler = Some(sender);
    }

    pub fn get_tcp_handler_clone(&self) -> TcpHandler {
        self.tcp_handler.clone().unwrap()
    }

    ///检查玩家是否已经在房间里
    pub fn check_player(&self, user_id: &u32) -> bool {
        self.player_room.contains_key(user_id)
    }

    pub fn get_room_id(&self, user_id: &u32) -> Option<u32> {
        let res = self.player_room.get(user_id);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let (_, room_id) = tools::binary::separate_long_2_int(*res);
        return Some(room_id);
    }

    ///执行函数，通过packet拿到cmd，然后从cmdmap拿到函数指针调用
    pub fn invok(&mut self, packet: Packet) {
        let cmd = packet.get_cmd();
        let f = self.cmd_map.get_mut(&cmd);
        if f.is_none() {
            warn!("there is no handler of cmd:{:?}!", cmd);
            return;
        }
        let _ = f.unwrap()(self, packet);
    }

    pub fn get_room_mut_by_user_id(&mut self, user_id: &u32) -> Option<&mut Room> {
        let res = self.player_room.get(user_id);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let (room_type, room_id) = tools::binary::separate_long_2_int(*res);
        let room_type = RoomType::try_from(room_type as u8);
        if let Err(e) = room_type {
            error!("{:?}", e);
            return None;
        }
        let room_type = room_type.unwrap();

        let room = match room_type {
            RoomType::OneVOneVOneVOneCustom => self.custom_room.get_room_mut(&room_id),
            RoomType::OneVOneVOneVOneMatch => self.match_room.get_room_mut(&room_id),
            RoomType::WorldBossCustom => self.world_boss_custom_room.get_room_mut(&room_id),
            RoomType::WorldBoseMatch => self.world_boss_match_room.get_room_mut(&room_id),
            _ => None,
        };
        room
    }

    #[allow(dead_code)]
    pub fn get_room_ref_by_user_id(&self, user_id: &u32) -> Option<&Room> {
        let res = self.player_room.get(user_id);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let (room_type, room_id) = tools::binary::separate_long_2_int(*res);
        let room_type = RoomType::try_from(room_type as u8);
        if let Err(e) = room_type {
            error!("{:?}", e);
            return None;
        }
        let room_type = room_type.unwrap();

        let room = match room_type {
            RoomType::OneVOneVOneVOneCustom => self.custom_room.get_room_ref(&room_id),
            RoomType::OneVOneVOneVOneMatch => self.match_room.get_room_ref(&room_id),
            RoomType::WorldBossCustom => self.world_boss_custom_room.get_room_ref(&room_id),
            RoomType::WorldBoseMatch => self.world_boss_match_room.get_room_ref(&room_id),
            _ => None,
        };
        room
    }

    ///命令初始化
    fn cmd_init(&mut self) {
        //热更静态配置
        self.cmd_map
            .insert(ServerCommonCode::ReloadTemps.into_u32(), reload_temps);
        //更新赛季信息
        self.cmd_map
            .insert(RoomCode::UpdateSeasonPush.into_u32(), update_season);
        //离线
        self.cmd_map.insert(RoomCode::OffLine.into_u32(), off_line);
        //离开房间
        self.cmd_map
            .insert(RoomCode::LeaveRoom.into_u32(), leave_room);
        //创建房间
        self.cmd_map
            .insert(RoomCode::CreateRoom.into_u32(), create_room);
        //换队伍
        self.cmd_map
            .insert(RoomCode::ChangeTeam.into_u32(), change_team);
        //T人
        self.cmd_map.insert(RoomCode::Kick.into_u32(), kick_member);
        //准备与取消
        self.cmd_map
            .insert(RoomCode::PrepareCancel.into_u32(), prepare_cancel);

        //添加房间
        self.cmd_map
            .insert(RoomCode::JoinRoom.into_u32(), join_room);
        //匹配房间
        self.cmd_map
            .insert(RoomCode::SearchRoom.into_u32(), search_room);
        //取消匹配房间
        self.cmd_map
            .insert(RoomCode::CancelSearch.into_u32(), cancel_search_room);
        //房间设置
        self.cmd_map
            .insert(RoomCode::RoomSetting.into_u32(), room_setting);
        //选择角色
        self.cmd_map
            .insert(RoomCode::ChoiceCharacter.into_u32(), choose_character);
        //选择技能
        self.cmd_map
            .insert(RoomCode::ChoiceSkill.into_u32(), choice_skills);
        //发送表情
        self.cmd_map.insert(RoomCode::Emoji.into_u32(), emoji);
        //确认进入房间
        self.cmd_map
            .insert(RoomCode::ConfirmIntoRoom.into_u32(), confirm_into_room);
        //结算处理
        self.cmd_map.insert(RoomCode::Summary.into_u32(), summary);
        //开始游戏
        self.cmd_map.insert(RoomCode::StartGame.into_u32(), start);
        //战斗服通知T人
        self.cmd_map
            .insert(RoomCode::BattleKickMember.into_u32(), battle_kick_member);
        //选择ai
        self.cmd_map
            .insert(RoomCode::ChoiceAI.into_u32(), choice_ai);
        //选择ai
        self.cmd_map
            .insert(RoomCode::UpdateWorldBossPush.into_u32(), update_worldboss);
    }
}
