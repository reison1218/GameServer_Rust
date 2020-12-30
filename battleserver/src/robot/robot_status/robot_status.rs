use super::*;
///pos操作类型
#[derive(Debug, Copy, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RobotStatus {
    None = 0,
}

impl RobotStatus {
    pub fn into_u8(self) -> u8 {
        let value: u8 = self.into();
        value
    }
}

impl Default for RobotStatus {
    fn default() -> Self {
        RobotStatus::None
    }
}
