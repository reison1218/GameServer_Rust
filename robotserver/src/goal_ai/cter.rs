use crate::goal_ai::goal::{Goal, NoneGoal};
use crate::goal_ai::goal_status::GoalStatus;
use crate::goal_ai::goal_think::GoalThink;
use crate::goal_ai::target_system::TargetingSystem;
use crossbeam::atomic::AtomicCell;
use std::borrow::Borrow;

pub struct Cter {
    pub target_system: TargetingSystem,
    pub goal_think: GoalThink,
    pub id: AtomicCell<u32>,             //角色唯一id
    pub goal: AtomicCell<Box<dyn Goal>>, //目标
}

impl Cter {
    pub fn udpate(&self) {
        self.goal_think.process(self);
    }

    pub fn activate(&self) {}

    pub fn get_goal_think(&self) -> &GoalThink {
        self.goal_think.borrow()
    }
}
