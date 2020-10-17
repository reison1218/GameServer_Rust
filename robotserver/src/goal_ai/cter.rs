use crate::goal_ai::goal::Goal;
use crate::goal_ai::goal_status::GoalStatus;
use crate::goal_ai::goal_think::GoalThink;
use crate::goal_ai::target_system::TargetingSystem;
use crossbeam::atomic::AtomicCell;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use std::borrow::Borrow;
use tools::get_mut_ref;
use tools::macros::GetMutRef;

///pos操作类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum CterStatus {
    None = 0,
    Alive = 1,
}

impl Default for CterStatus {
    fn default() -> Self {
        CterStatus::None
    }
}

impl CterStatus {
    pub fn into_u32(self) -> u32 {
        let res: u8 = self.into();
        res as u32
    }

    pub fn into_u8(self) -> u8 {
        let res: u8 = self.into();
        res
    }
}

#[derive(Default)]
pub struct Cter {
    pub statuc: CterStatus,
    pub target_system: TargetingSystem,
    pub goal_think: GoalThink,
    pub id: AtomicCell<u32>,         //角色唯一id
    pub goal: Option<Box<dyn Goal>>, //目标
}

get_mut_ref!(Cter);

impl Cter {
    pub fn udpate(&self) {
        self.goal_think.process(self);
    }

    pub fn activate(&self) {}

    pub fn get_goal_think(&self) -> &GoalThink {
        self.goal_think.borrow()
    }
}
