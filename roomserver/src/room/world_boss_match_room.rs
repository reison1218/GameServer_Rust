use super::member::Member;
use super::room::RoomState;
use super::room::MEMBER_MAX;
use super::room_model::RoomSetting;
use super::room_model::RoomType;
use super::{
    room::Room,
    room_model::{RoomCache, RoomModel},
};
use crate::room::room::recycle_room_id;
use crate::task_timer::build_confirm_into_room_task;
use crate::task_timer::Task;
use crossbeam::channel::Sender;
use log::error;
use log::info;
use rayon::slice::ParallelSliceMut;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use tools::tcp_message_io::TcpHandler;
///worldboss匹配房结构体
#[derive(Clone, Default)]
pub struct WorldBossMatchRoom {
    pub rooms: HashMap<u32, Room>,  //key:房间id    value:房间结构体
    pub room_cache: Vec<RoomCache>, //key:房间id    value:房间人数
}

impl WorldBossMatchRoom {
    pub fn get_room_mut(&mut self, room_id: &u32) -> Option<&mut Room> {
        self.rooms.get_mut(room_id)
    }

    pub fn get_room_cache_mut(&mut self, room_id: &u32) -> Option<&mut RoomCache> {
        let res = self.room_cache.iter_mut().find(|x| x.room_id == *room_id);
        res
    }

    ///删除缓存房间
    pub fn remove_room_cache(&mut self, room_id: &u32) {
        let index = self
            .room_cache
            .iter()
            .enumerate()
            .find(|(_, room)| room.room_id == *room_id);

        if index.is_none() {
            return;
        }
        let (index, _) = index.unwrap();
        self.room_cache.remove(index);
        //重新排序
        self.room_cache.par_sort_by(|a, b| b.count.cmp(&a.count));
    }

    ///快速加入
    pub fn quickly_start(
        &mut self,
        member: Member,
        sender: TcpHandler,
        task_sender: Sender<Task>,
    ) -> anyhow::Result<u32> {
        let room_id: u32;
        //如果房间缓存里没有，则创建新房间
        if self.room_cache.is_empty() {
            //创建房间
            room_id = self.create_room(member, None, sender, task_sender)?;
        } else {
            //如果有，则往房间里塞
            room_id = self.get_room_cache_last_room_id()?;
            //将成员加进房间
            let room_mut = self.get_mut_room_by_room_id(&room_id)?;
            if room_mut.get_member_count() >= MEMBER_MAX {
                anyhow::bail!("room is None,room_id:{}", room_id)
            }
            let user_id = member.user_id;
            //将成员加入到房间中
            room_mut.add_member(member, None)?;
            //解决房间队列缓存
            let room_cache = self.room_cache.get_mut(0).unwrap();
            //cache人数加1
            room_cache.count += 1;
            let room_cache_count = room_cache.count as usize;
            //如果人满里，则从缓存房间列表中弹出
            if room_cache_count >= MEMBER_MAX {
                //人满了就从队列里面弹出去
                self.remove_room_cache(&room_id);
                info!("匹配房人满,将房间从匹配队列移除！room_id:{}", room_id);
                //推送匹配成功通知
                let room_mut = self.rooms.get_mut(&room_id).unwrap();
                room_mut.push_match_success();
                //创建检测进入房间延迟任务
                build_confirm_into_room_task(RoomType::WorldBoseMatch, room_id, task_sender);
            }
            //重新排序
            self.room_cache.par_sort_by(|a, b| b.count.cmp(&a.count));
            let match_room_count = self.room_cache.len();
            info!(
                "玩家匹配到房间！当前房间人数：{},match_user_id:{},room_id:{}",
                room_cache_count, user_id, room_id
            );
            info!("当前匹配房数量:{}!", match_room_count);
        }
        Ok(room_id)
    }

    fn get_room_cache_last_room_id(&self) -> anyhow::Result<u32> {
        let room_cache = self.room_cache.get(0);
        if room_cache.is_none() {
            let str = "room_cache is empty!".to_owned();
            error!("{:?}", str.as_str());
            anyhow::bail!("{:?}", str)
        }
        let room_id = room_cache.unwrap().room_id;
        Ok(room_id)
    }
}

impl RoomModel for WorldBossMatchRoom {
    fn get_room_type(&self) -> RoomType {
        RoomType::WorldBoseMatch
    }

    fn get_room_mut(&mut self, room_id: &u32) -> Option<&mut Room> {
        let res = self.rooms.get_mut(room_id);
        if res.is_none() {
            return None;
        }
        let room = res.unwrap();
        Some(room)
    }

    fn get_room_ref(&self, room_id: &u32) -> Option<&Room> {
        let res = self.rooms.get(room_id);
        if res.is_none() {
            return None;
        }
        let room = res.unwrap();
        Some(room)
    }

    fn create_room(
        &mut self,
        owner: Member,
        room_setting: Option<RoomSetting>,
        sender: TcpHandler,
        task_sender: Sender<Task>,
    ) -> anyhow::Result<u32> {
        let mut room = Room::new(owner, RoomType::WorldBoseMatch, sender, task_sender)?;
        let room_id = room.get_room_id();
        //加入worldboss
        unsafe {
            let world_boss_id = crate::WORLD_BOSS.world_boss_id as u32;
            let worldboss_temp = crate::TEMPLATES
                .worldboss_temp_mgr()
                .temps
                .get(&world_boss_id)
                .unwrap();

            let member = Member::new_for_robot(worldboss_temp.robot_id, 2);
            room.robots.insert(member.get_user_id());
            let _ = room.add_member(member, Some(3));
        }
        self.rooms.insert(room_id, room);
        let mut rc = RoomCache::default();
        rc.room_id = room_id;
        rc.count = 2;
        self.room_cache.push(rc);
        self.room_cache.par_sort_by(|a, b| b.count.cmp(&a.count));
        Ok(room_id)
    }

    fn leave_room(
        &mut self,
        notice_type: u8,
        room_id: &u32,
        user_id: &u32,
        need_push_self: bool,
        need_punish: bool,
    ) -> anyhow::Result<u32> {
        let room = self.get_mut_room_by_room_id(room_id)?;
        let room_id = *room_id;
        //其他状态房间服自行处理
        if need_punish {
            //处理匹配惩罚,如果是匹配放，并且当前房间是满的，则进行惩罚
            //room.check_punish_for_leave(*user_id);
        }
        room.remove_member(notice_type, user_id, need_push_self);
        //改变房间状态
        room.state = RoomState::AwaitConfirm;
        let need_remove = room.is_empty();
        let now_count = room.get_member_count();
        let mut need_add_cache = false;
        //如果房间之前是满都，就给所有人取消准备
        if room.get_state() == RoomState::AwaitConfirm && now_count < MEMBER_MAX {
            room.do_cancel_prepare();
            need_add_cache = true;
        }

        if need_remove {
            return Ok(room_id);
        }

        let room_cache = self.get_room_cache_mut(&room_id);
        if let Some(room_cache) = room_cache {
            room_cache.count -= 1;
            //重新排序
            self.room_cache.par_sort_by(|a, b| b.count.cmp(&a.count));
        } else if room_cache.is_none() && need_add_cache {
            let mut rc = RoomCache::default();
            rc.room_id = room_id;
            rc.count = now_count as u8;
            self.room_cache.push(rc);
            //重新排序
            self.room_cache.par_sort_by(|a, b| b.count.cmp(&a.count));
            info!(
                "玩家离开房间匹配房间，满足条件，将放进重新放入匹配队列,room_id:{}",
                room_id
            );
        }
        Ok(room_id)
    }

    fn rm_room(&mut self, room_id: &u32) -> Option<Room> {
        let res = self.rooms.remove(room_id);
        self.remove_room_cache(room_id);
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
