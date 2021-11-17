use std::{
    borrow::BorrowMut,
    collections::{hash_map::RandomState, HashMap},
};

use log::info;
use protobuf::Message;
use tools::{cmd_code::ClientCode, net_message_io::NetHandler, protos::room::S_LEAVE_ROOM};

use crate::{room::room::recycle_room_id, task_timer::Task};

use super::{
    member::Member,
    room::{Room, RoomState},
    room_model::{RoomModel, RoomSetting, RoomType},
};

///好友房结构体
#[derive(Clone, Default)]
pub struct CustomRoom {
    pub rooms: HashMap<u32, Room>, //封装房间房间id->房间结构体实例
}

impl RoomModel for CustomRoom {
    fn get_room_type(&self) -> RoomType {
        RoomType::OneVOneVOneVOneCustom
    }

    fn get_room_mut(&mut self, room_id: &u32) -> Option<&mut Room> {
        let res = self.rooms.get_mut(room_id);
        res
    }

    fn get_room_ref(&self, room_id: &u32) -> Option<&Room> {
        let res = self.rooms.get(room_id);
        res
    }

    ///创建房间
    fn create_room(
        &mut self,
        owner: Member,
        room_setting: Option<RoomSetting>,
        sender: NetHandler,
        task_sender: crossbeam::channel::Sender<Task>,
    ) -> anyhow::Result<u32> {
        let user_id = owner.user_id;
        let mut room = Room::new(owner, RoomType::OneVOneVOneVOneCustom, sender, task_sender)?;
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

    ///离开房间
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

    fn get_rooms_mut(&mut self) -> &mut HashMap<u32, Room, RandomState> {
        self.rooms.borrow_mut()
    }
}
