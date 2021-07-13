use crate::room::member::Member;
use crate::room::room::Room;
use crate::task_timer::Task;
use crate::TEMPLATES;
use log::error;
use log::warn;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use std::str::FromStr;
use tools::protos::base::RoomSettingPt;
use tools::tcp_message_io::TcpHandler;

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
    None = 0,                  //无效
    OneVOneVOneVOneCustom = 1, //1v1v1v1自定义房间
    OneVOneVOneVOneMatch = 2,  //1v1v1v1匹配房间
    WorldBossCustom = 3,       //世界boss自定义房间
    WorldBoseMatch = 4,        //世界boss匹配房间
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
#[derive(Debug, Copy, Clone)]
pub struct RoomSetting {
    pub turn_limit_time: u32, //回合限制时间
    pub season_id: i32,       //赛季id
    pub ai_level: u8,         //ai等级
}

impl Default for RoomSetting {
    fn default() -> Self {
        let temp = TEMPLATES
            .constant_temp_mgr()
            .temps
            .get("battle_turn_limit_time");
        let turn_limit_time;
        match temp {
            Some(temp) => {
                let res = u32::from_str(temp.value.as_str());

                match res {
                    Ok(res) => {
                        turn_limit_time = res;
                    }
                    Err(err) => {
                        error!("{:?}", err);
                        turn_limit_time = 120000;
                    }
                }
            }

            None => {
                turn_limit_time = 120000;
                warn!("constant temp's battle_turn_limit_time is none!")
            }
        }
        RoomSetting {
            season_id: 0,
            ai_level: 3,
            turn_limit_time,
        }
    }
}

impl From<&RoomSettingPt> for RoomSetting {
    fn from(rs_pt: &RoomSettingPt) -> Self {
        let ai_level = rs_pt.ai_level as u8;
        let turn_limit_time = rs_pt.turn_limit_time;
        let season_id = rs_pt.season_id;
        let rs = RoomSetting {
            turn_limit_time,
            season_id,
            ai_level,
        };
        rs
    }
}

impl From<&RoomSetting> for RoomSettingPt {
    fn from(r: &RoomSetting) -> Self {
        let mut rsp = RoomSettingPt::new();
        rsp.set_season_id(r.season_id);
        rsp.set_turn_limit_time(r.turn_limit_time);
        rsp.set_ai_level(r.ai_level as u32);
        rsp
    }
}

///房间缓存
#[derive(Debug, Copy, Clone, Default)]
pub struct RoomCache {
    pub room_id: u32,
    pub count: u8,
}

pub trait RoomModel {
    fn get_room_type(&self) -> RoomType;

    fn get_room_mut(&mut self, room_id: &u32) -> Option<&mut Room>;

    fn get_room_ref(&self, room_id: &u32) -> Option<&Room>;

    fn create_room(
        &mut self,
        owner: Member,
        room_setting: Option<RoomSetting>,
        sender: TcpHandler,
        task_sender: crossbeam::channel::Sender<Task>,
    ) -> anyhow::Result<u32>;

    fn leave_room(
        &mut self,
        notice_type: u8,
        room_id: &u32,
        user_id: &u32,
        need_push_self: bool,
        need_punish: bool,
    ) -> anyhow::Result<u32>;

    fn rm_room(&mut self, room_id: &u32) -> Option<Room>;

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
