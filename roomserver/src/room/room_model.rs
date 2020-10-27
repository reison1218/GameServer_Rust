use crate::robot::robot_task_mgr::RobotTask;
use crate::room::member::Member;
use crate::room::room::{MemberLeaveNoticeType, RoomState};
use crate::room::room::{Room, MEMBER_MAX};
use crate::task_timer::{Task, TaskCmd};
use crate::TEMPLATES;
use log::{error, info, warn};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use protobuf::Message;
use rayon::slice::ParallelSliceMut;
use serde_json::{Map, Value};
use std::borrow::BorrowMut;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::str::FromStr;
use tools::cmd_code::ClientCode;
use tools::protos::base::RoomSettingPt;
use tools::protos::room::S_LEAVE_ROOM;
use tools::tcp::TcpSender;
use tools::templates::template::TemplateMgrTrait;
use tools::templates::tile_map_temp::TileMapTempMgr;

///teamID枚举
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum TeamId {
    Min = 1, //最小teamid
    Max = 4, //最大teamid
}

///房间类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RoomType {
    None = 0,         //无效
    Custom = 1,       //自定义房间
    Match = 2,        //匹配房间
    SeasonPve = 3,    //赛季PVE房间
    WorldBossPve = 4, //世界boss房间
}

impl RoomType {
    pub fn into_u8(self) -> u8 {
        let res: u8 = self.into();
        res
    }

    pub fn into_u32(self) -> u32 {
        let res: u8 = self.into();
        res as u32
    }
}

///房间设置
#[derive(Debug, Copy, Clone, Default)]
pub struct RoomSetting {
    pub turn_limit_time: u32,   //回合限制时间
    pub season_id: u32,         //赛季id
    pub is_open_ai: bool,       //是否开启ai
    pub victory_condition: u32, //胜利条件
}

impl From<RoomSettingPt> for RoomSetting {
    fn from(rs_pt: RoomSettingPt) -> Self {
        let is_open_ai = rs_pt.is_open_ai;
        let victory_condition = rs_pt.victory_condition;
        let turn_limit_time = rs_pt.turn_limit_time;
        let season_id = rs_pt.season_id;
        let rs = RoomSetting {
            turn_limit_time,
            season_id,
            is_open_ai,
            victory_condition,
        };
        rs
    }
}

impl From<RoomSetting> for RoomSettingPt {
    fn from(r: RoomSetting) -> Self {
        let mut rsp = RoomSettingPt::new();
        rsp.set_victory_condition(r.victory_condition);
        rsp.set_season_id(r.season_id);
        rsp.set_turn_limit_time(r.turn_limit_time);
        rsp.set_is_open_ai(r.is_open_ai);
        rsp
    }
}

///房间缓存
#[derive(Debug, Copy, Clone, Default)]
pub struct RoomCache {
    room_id: u32,
    count: u8,
}

pub trait RoomModel {
    fn get_room_type(&self) -> RoomType;

    fn get_room_mut(&mut self, room_id: &u32) -> Option<&mut Room>;

    fn create_room(
        &mut self,
        owner: Member,
        sender: TcpSender,
        task_sender: crossbeam::channel::Sender<Task>,
        robot_sender: crossbeam::channel::Sender<RobotTask>,
    ) -> anyhow::Result<u32>;
    fn leave_room(&mut self, notice_type: u8, room_id: &u32, user_id: &u32) -> anyhow::Result<u32>;

    fn rm_room(&mut self, room_id: &u32);

    fn get_rooms_mut(&mut self) -> &mut HashMap<u32, Room>;

    ///根据房间id获得房间的可变指针
    fn get_mut_room_by_room_id(&mut self, room_id: &u32) -> anyhow::Result<&mut Room> {
        let res = self.get_rooms_mut().get_mut(room_id);
        if res.is_none() {
            let s = format!("this room is not exit！room_id:{}", room_id);
            anyhow::bail!(s)
        }
        Ok(res.unwrap())
    }

    ///根据房间id获得房间的只读指针
    fn get_ref_room_by_room_id(&mut self, room_id: &u32) -> anyhow::Result<&Room> {
        let res = self.get_rooms_mut().get(room_id);
        if res.is_none() {
            anyhow::bail!("this room is not exit,room_id:{}", room_id)
        }
        Ok(res.unwrap())
    }
}

///好友房结构体
#[derive(Clone, Default)]
pub struct CustomRoom {
    pub rooms: HashMap<u32, Room>, //封装房间房间id->房间结构体实例
}

impl RoomModel for CustomRoom {
    fn get_room_type(&self) -> RoomType {
        RoomType::Custom
    }

    fn get_room_mut(&mut self, room_id: &u32) -> Option<&mut Room> {
        let res = self.rooms.get_mut(room_id);
        res
    }

    ///创建房间
    fn create_room(
        &mut self,
        owner: Member,
        sender: TcpSender,
        task_sender: crossbeam::channel::Sender<Task>,
        robot_sender: crossbeam::channel::Sender<RobotTask>,
    ) -> anyhow::Result<u32> {
        let user_id = owner.user_id;
        let mut room = Room::new(
            owner.clone(),
            RoomType::Custom,
            sender,
            task_sender,
            robot_sender,
        )?;
        let room_id = room.get_room_id();
        self.rooms.insert(room_id, room);
        let room = self.rooms.get_mut(&room_id).unwrap();
        //同志房间其他成员
        room.room_add_member_notice(&user_id);
        Ok(room_id)
    }

    ///离开房间
    fn leave_room(&mut self, notice_type: u8, room_id: &u32, user_id: &u32) -> anyhow::Result<u32> {
        let room = self.get_mut_room_by_room_id(room_id)?;
        room.remove_member(notice_type, user_id);
        let mut slr = S_LEAVE_ROOM::new();
        slr.set_is_succ(true);
        room.send_2_client(
            ClientCode::LeaveRoom,
            *user_id,
            slr.write_to_bytes().unwrap(),
        );
        let room_id = room.get_room_id();
        Ok(room_id)
    }

    fn rm_room(&mut self, room_id: &u32) {
        self.rooms.remove(room_id);
        info!(
            "删除房间，释放内存！room_type:{:?},room_id:{}",
            self.get_room_type(),
            room_id
        );
    }

    fn get_rooms_mut(&mut self) -> &mut HashMap<u32, Room, RandomState> {
        self.rooms.borrow_mut()
    }
}

///匹配房结构体
#[derive(Clone, Default)]
pub struct MatchRoom {
    pub rooms: HashMap<u32, Room>,  //key:房间id    value:房间结构体
    pub room_cache: Vec<RoomCache>, //key:房间id    value:房间人数
}

impl RoomModel for MatchRoom {
    fn get_room_type(&self) -> RoomType {
        RoomType::Match
    }

    fn get_room_mut(&mut self, room_id: &u32) -> Option<&mut Room> {
        let res = self.rooms.get_mut(room_id);
        if res.is_none() {
            return None;
        }
        let room = res.unwrap();
        Some(room)
    }

    ///创建房间
    fn create_room(
        &mut self,
        owner: Member,
        sender: TcpSender,
        task_sender: crossbeam::channel::Sender<Task>,
        robot_sender: crossbeam::channel::Sender<RobotTask>,
    ) -> anyhow::Result<u32> {
        let mut room = Room::new(owner, RoomType::Match, sender, task_sender, robot_sender)?;
        let room_id = room.get_room_id();
        self.rooms.insert(room_id, room);
        let mut rc = RoomCache::default();
        rc.room_id = room_id;
        rc.count = 1;
        self.room_cache.push(rc);
        Ok(room_id)
    }

    ///离开房间
    fn leave_room(&mut self, notice_type: u8, room_id: &u32, user_id: &u32) -> anyhow::Result<u32> {
        let room = self.get_mut_room_by_room_id(room_id)?;
        let room_id = *room_id;
        let member_count = room.get_member_count();
        room.remove_member(notice_type, user_id);
        let need_remove = room.is_empty();
        let now_count = room.get_member_count();
        let mut need_add_cache = false;
        //如果房间之前是满都，就给所有人取消准备
        if room.get_state() == RoomState::Await
            && member_count == MEMBER_MAX as usize
            && now_count < member_count
        {
            let map = room.members.clone();
            for id in map.keys() {
                room.prepare_cancel(id, false);
            }
            if room.get_state() == RoomState::Await {
                need_add_cache = true;
            }
        }

        if need_remove {
            return Ok(room_id);
        }

        let room_cache = self.get_room_cache_mut(&room_id);
        if room_cache.is_some() {
            let rc = room_cache.unwrap();
            rc.count -= 1;
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

    ///删除房间
    fn rm_room(&mut self, room_id: &u32) {
        self.rooms.remove(room_id);
        self.remove_room_cache(room_id);
        info!(
            "删除房间，释放内存！room_type:{:?},room_id:{}",
            self.get_room_type(),
            room_id
        );
    }

    fn get_rooms_mut(&mut self) -> &mut HashMap<u32, Room, RandomState> {
        self.rooms.borrow_mut()
    }
}

impl MatchRoom {
    pub fn get_room_mut(&mut self, room_id: &u32) -> Option<&mut Room> {
        self.rooms.get_mut(room_id)
    }

    ///离开房间，离线也好，主动离开也好
    pub fn leave(&mut self, room_id: u32, user_id: &u32) -> anyhow::Result<u32> {
        let res = self.leave_room(MemberLeaveNoticeType::Leave as u8, &room_id, user_id);
        res
    }

    pub fn get_room_cache_mut(&mut self, room_id: &u32) -> Option<&mut RoomCache> {
        let res = self.room_cache.iter_mut().find(|x| x.room_id == *room_id);
        res
    }

    ///删除缓存房间
    pub fn remove_room_cache(&mut self, room_id: &u32) {
        let mut index = -1_isize;
        for i in self.room_cache.iter() {
            index += 1;
            if i.room_id != *room_id {
                continue;
            }
            break;
        }
        if index < 0 {
            return;
        }
        self.room_cache.remove(index as usize);
    }

    ///快速加入
    pub fn quickly_start(
        &mut self,
        member: Member,
        sender: TcpSender,
        task_sender: crossbeam::channel::Sender<Task>,
        robot_sender: crossbeam::channel::Sender<RobotTask>,
    ) -> anyhow::Result<u32> {
        let room_id: u32;
        let user_id = member.user_id;
        //如果房间缓存里没有，则创建新房间
        if self.room_cache.is_empty() {
            //校验地图配置
            let room_tmp_ref: &TileMapTempMgr = TEMPLATES.get_tile_map_temp_mgr_ref();
            if room_tmp_ref.is_empty() {
                anyhow::bail!("TileMapTempMgr is None")
            }
            //创建房间
            room_id = self.create_room(member, sender, task_sender, robot_sender)?;
            info!("创建匹配房间,room_id:{},user_id:{}", room_id, user_id);
        } else {
            //如果有，则往房间里塞
            room_id = self.get_room_cache_last_room_id()?;
            //将成员加进房间
            let room_mut = self.get_mut_room_by_room_id(&room_id)?;
            if room_mut.get_member_count() >= MEMBER_MAX as usize {
                anyhow::bail!("room is None,room_id:{}", room_id)
            }
            //将成员加入到房间中
            room_mut.add_member(member)?;
            //解决房间队列缓存
            let room_cache_array: &mut Vec<RoomCache> = self.room_cache.as_mut();
            let room_cache = room_cache_array.last_mut().unwrap();
            //cache人数加1
            room_cache.count += 1;
            //如果人满里，则从缓存房间列表中弹出
            if room_cache.count >= MEMBER_MAX {
                room_cache_array.pop();
                info!("匹配房人满,将房间从匹配队列移除！room_id:{}", room_id);
                //创建延迟任务，并发送给定时器接收方执行
                let mut task = Task::default();
                let time_limit = TEMPLATES
                    .get_constant_temp_mgr_ref()
                    .temps
                    .get("kick_not_prepare_time");
                if let Some(time) = time_limit {
                    let time = u64::from_str(time.value.as_str())?;
                    task.delay = time + 500;
                } else {
                    task.delay = 60000_u64;
                    warn!("the Constant kick_not_prepare_time is None!pls check!");
                }

                task.cmd = TaskCmd::MatchRoomStart as u16;
                let mut map = Map::new();
                map.insert("room_id".to_owned(), Value::from(room_id));
                task.data = Value::from(map);
                let res = task_sender.send(task);
                if let Err(e) = res {
                    error!("{:?}", e);
                }
            }
            //重新排序
            room_cache_array.par_sort_by(|a, b| b.count.cmp(&a.count));
        }
        Ok(room_id)
    }

    fn get_room_cache_last_room_id(&self) -> anyhow::Result<u32> {
        let room_cache = self.room_cache.last();
        if room_cache.is_none() {
            let str = "room_cache is empty!".to_owned();
            error!("{:?}", str.as_str());
            anyhow::bail!("{:?}", str)
        }
        let room_id = room_cache.unwrap().room_id;
        Ok(room_id)
    }
}
