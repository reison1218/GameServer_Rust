use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;

///回合行为类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum BattleCterState {
    Alive = 0,
    Die = 1,
    OffLine = 2, //离线
}

impl Default for BattleCterState {
    fn default() -> Self {
        BattleCterState::Alive
    }
}

impl BattleCterState {
    pub fn into_u8(self) -> u8 {
        let value: u8 = self.into();
        value
    }
}

///攻击状态
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum AttackState {
    None = 0,   //无效
    Able = 1,   //有效
    Locked = 2, //锁定，不可攻击
}

impl Default for AttackState {
    fn default() -> Self {
        AttackState::None
    }
}

impl AttackState {
    pub fn into_u8(self) -> u8 {
        let value: u8 = self.into();
        value
    }
}

///回合行为类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RobotState {
    Idle = 0,    //空闲
    Working = 1, //工作
}

impl Default for RobotState {
    fn default() -> Self {
        RobotState::Idle
    }
}
