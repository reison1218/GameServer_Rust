use crate::goal_ai::goal::{CombinedGoal, Goal, NoneGoal};
use crate::goal_ai::goal_status::GoalStatus;
use crossbeam::atomic::AtomicCell;

pub struct Cter {
    pub id: AtomicCell<u32>, //角色唯一id
    pub goal: Box<dyn Goal>, //目标
}

impl Default for Box<dyn Goal> {
    fn default() -> Self {
        Box::new(NoneGoal::default())
    }
}

pub struct GoalThink {}

impl Goal for GoalThink {
    fn set_target_index(&self, target_index: u32) {
        unimplemented!()
    }

    fn calculate_expect(&self, cter: &Cter) -> u32 {
        unimplemented!()
    }

    fn activate(&self, cter: &Cter) {
        unimplemented!()
    }

    fn process(&self, cter: &Cter) -> GoalStatus {
        unimplemented!()
    }

    fn terminate(&self, cter: &Cter) {
        unimplemented!()
    }

    fn add_goal(&self, goal: Box<dyn Goal>) {
        unimplemented!()
    }

    fn get_goal_status(&self) -> GoalStatus {
        unimplemented!()
    }
}
