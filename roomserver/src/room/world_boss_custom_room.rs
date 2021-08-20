use std::{borrow::BorrowMut, collections::HashMap};

use log::info;
use protobuf::Message;
use tools::{cmd_code::ClientCode, protos::room::S_LEAVE_ROOM};

use crate::room::room::recycle_room_id;

use super::{
    member::Member,
    room::{Room, RoomState},
    room_model::{RoomModel, RoomType},
};

///世界boss自定义房间结构体
#[derive(Clone, Default)]
pub struct WorldBossCustomRoom {
    pub rooms: HashMap<u32, Room>, //封装房间房间id->房间结构体实例
}

impl RoomModel for WorldBossCustomRoom {
    fn get_room_type(&self) -> super::room_model::RoomType {
        RoomType::WorldBossCustom
    }

    fn get_room_mut(&mut self, room_id: &u32) -> Option<&mut Room> {
        self.rooms.get_mut(room_id)
    }

    fn get_room_ref(&self, room_id: &u32) -> Option<&Room> {
        self.rooms.get(room_id)
    }

    fn create_room(
        &mut self,
        owner: super::member::Member,
        room_setting: Option<super::room_model::RoomSetting>,
        sender: tools::tcp_message_io::TcpHandler,
        task_sender: crossbeam::channel::Sender<crate::task_timer::Task>,
    ) -> anyhow::Result<u32> {
        let user_id = owner.user_id;
        let mut room = Room::new(owner, RoomType::WorldBossCustom, sender, task_sender)?;
        //加入worldboss
        unsafe {
            let world_boss_id = crate::WORLD_BOSS.world_boss_id as u32;
            let worldboss_temp = crate::TEMPLATES
                .worldboss_temp_mgr()
                .temps
                .get(&world_boss_id)
                .unwrap();

            let member = Member::new_for_robot(worldboss_temp.robot_id, 2, Some(3));
            room.robots.insert(member.get_user_id());
            let _ = room.add_member(member, Some(3), false);
        }
        if let Some(room_setting) = room_setting {
            room.setting = room_setting;
        }

        let room_id = room.get_room_id();
        self.rooms.insert(room_id, room);
        let room = self.rooms.get_mut(&room_id).unwrap();
        //同志房间其他成员
        room.notice_new_member(user_id);
        Ok(room_id)
    }

    fn leave_room(
        &mut self,
        notice_type: u8,
        room_id: &u32,
        user_id: &u32,
        need_push_self: bool,
        _: bool,
    ) -> anyhow::Result<u32> {
        let room = self.get_mut_room_by_room_id(room_id)?;
        let room_id = room.get_room_id();
        room.remove_member(notice_type, user_id, need_push_self);
        if room.state == RoomState::ChoiceIndex {
            return Ok(room_id);
        }
        let mut slr = S_LEAVE_ROOM::new();
        slr.set_is_succ(true);
        room.send_2_client(
            ClientCode::LeaveRoom,
            *user_id,
            slr.write_to_bytes().unwrap(),
        );
        Ok(room_id)
    }

    fn rm_room(&mut self, room_id: &u32) -> Option<Room> {
        let res = self.rooms.remove(room_id);
        match res {
            Some(room) => {
                recycle_room_id(room.get_room_id());
                info!(
                    "删除房间，释放内存！room_type:{:?},room_id:{}",
                    room.get_room_type(),
                    room_id
                );
                Some(room)
            }
            None => None,
        }
    }

    fn get_rooms_mut(&mut self) -> &mut HashMap<u32, Room> {
        self.rooms.borrow_mut()
    }
}
