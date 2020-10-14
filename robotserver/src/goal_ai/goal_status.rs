use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

///pos操作类型
#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum GoalStatus {
    None = 0,   //无效值
    Idel = 1,   //闲置
    Active = 2, //激活
    Finish = 3, //完成
    Fail = 4,   //失败
}

impl Default for GoalStatus {
    fn default() -> Self {
        GoalStatus::None
    }
}

impl GoalStatus {
    pub fn into_u8(self) -> u8 {
        let value: u8 = self.into();
        value
    }
}
