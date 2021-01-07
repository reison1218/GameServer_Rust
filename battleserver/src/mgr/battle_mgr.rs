use crate::handlers::battle_handler::{
    action, choice_index, emoji, leave_room, pos, reload_temps, start, update_season,
};
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::room::Room;
use crate::task_timer::Task;
use crossbeam::channel::Sender;
use log::{info, warn};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use tools::cmd_code::{BattleCode, ServerCommonCode};
use tools::util::packet::Packet;

type CmdFn = HashMap<u32, fn(&mut BattleMgr, Packet) -> anyhow::Result<()>, RandomState>;

///战斗管理器
#[derive(Default)]
pub struct BattleMgr {
    pub player_room: HashMap<u32, u32>,               //玩家对应房间
    pub rooms: HashMap<u32, Room>,                    //房间map
    pub cmd_map: CmdFn,                               //命令函数指针
    pub game_center_channel: Option<Sender<Vec<u8>>>, //tcp客户的
    pub task_sender: Option<Sender<Task>>,            //task channel的发送方
    pub robot_task_sender: Option<Sender<RobotTask>>, //机器人task channel的发送方
}

tools::get_mut_ref!(BattleMgr);

impl BattleMgr {
    pub fn set_game_center_channel(&mut self, ts: Sender<Vec<u8>>) {
        self.game_center_channel = Some(ts);
    }

    pub fn new() -> BattleMgr {
        let mut bm = BattleMgr::default();
        bm.cmd_init();
        bm
    }

    pub fn get_game_center_channel_clone(&self) -> crossbeam::channel::Sender<Vec<u8>> {
        self.game_center_channel.as_ref().unwrap().clone()
    }

    pub fn get_task_sender_clone(&self) -> crossbeam::channel::Sender<Task> {
        self.task_sender.as_ref().unwrap().clone()
    }

    pub fn get_robot_task_sender_clone(&self) -> crossbeam::channel::Sender<RobotTask> {
        self.robot_task_sender.as_ref().unwrap().clone()
    }

    pub fn send_2_server(&mut self, cmd: u32, user_id: u32, bytes: Vec<u8>) {
        let bytes = Packet::build_packet_bytes(cmd, user_id, bytes, true, false);
        let res = self.get_game_center_channel_mut();
        let size = res.send(bytes);
        if let Err(e) = size {
            warn!("{:?}", e);
        }
    }

    pub fn get_game_center_channel_mut(&mut self) -> &mut Sender<Vec<u8>> {
        self.game_center_channel.as_mut().unwrap()
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

    pub fn get_room_mut(&mut self, user_id: &u32) -> Option<&mut Room> {
        let res = self.player_room.get(user_id);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let room_id = *res;
        self.rooms.get_mut(&room_id)
    }

    #[allow(dead_code)]
    pub fn get_room_ref(&self, user_id: &u32) -> Option<&Room> {
        let res = self.player_room.get(user_id);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let room_id = *res;
        self.rooms.get(&room_id)
    }

    ///删除房间
    pub fn rm_room(&mut self, room_id: u32) {
        let room = self.rooms.remove(&room_id);
        if let Some(room) = room {
            let room_type = room.get_room_type();
            let room_id = room.get_room_id();
            for user_id in room.members.keys() {
                self.player_room.remove(user_id);
            }
            info!(
                "删除房间，释放内存！room_type:{:?},room_id:{}",
                room_type, room_id
            );
        }
    }

    ///命令初始化
    fn cmd_init(&mut self) {
        //掉线
        self.cmd_map
            .insert(ServerCommonCode::LineOff.into_u32(), leave_room);
        //离开房间
        self.cmd_map
            .insert(ServerCommonCode::LeaveRoom.into_u32(), leave_room);
        //更新赛季信息
        self.cmd_map
            .insert(ServerCommonCode::UpdateSeason.into_u32(), update_season);
        //热更静态配置
        self.cmd_map
            .insert(ServerCommonCode::ReloadTemps.into_u32(), reload_temps);
        //开始战斗
        self.cmd_map.insert(BattleCode::Start.into_u32(), start);

        //发送表情
        self.cmd_map.insert(BattleCode::Emoji.into_u32(), emoji);

        //选择占位
        self.cmd_map
            .insert(BattleCode::ChoiceIndex.into_u32(), choice_index);
        //------------------------------------以下是战斗相关的--------------------------------
        //请求行动
        self.cmd_map.insert(BattleCode::Action.into_u32(), action);

        //请求pos
        self.cmd_map.insert(BattleCode::Pos.into_u32(), pos);
    }
}
