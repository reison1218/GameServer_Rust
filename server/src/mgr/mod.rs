pub mod game_mgr;
pub mod timer_mgr;

use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
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
