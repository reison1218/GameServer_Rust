pub mod game_mgr;
pub mod timer_mgr;

use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
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
