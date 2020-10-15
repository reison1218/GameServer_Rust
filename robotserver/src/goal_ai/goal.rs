use crate::goal_ai::cter::Cter;
use crate::goal_ai::goal_combined::{GoalCombined, GoalCombinedTest};
use crate::goal_ai::goal_status::GoalStatus;
use crossbeam::atomic::AtomicCell;
use crossbeam::queue::ArrayQueue;
use std::borrow::Borrow;

///目标trait
pub trait Goal: Send + 'static {
    ///激活
    fn activate(&self, cter: &Cter);

    ///推进
    fn process(&self, cter: &Cter) -> GoalStatus;

    ///终止
    fn terminate(&self);

    ///获得目标状态
    fn get_goal_status(&self) -> GoalStatus;

    ///是否激活
    fn is_active(&self) -> bool {
        self.get_goal_status() == GoalStatus::Active
    }

    ///是否完成
    fn is_finished(&self) -> bool {
        self.get_goal_status() == GoalStatus::Finish
    }

    ///是否失败
    fn is_failed(&self) -> bool {
        self.get_goal_status() == GoalStatus::Fail
    }
}

impl Default for Box<dyn Goal> {
    fn default() -> Self {
        Box::new(NoneGoal::default())
    }
}

#[derive(Default)]
pub struct NoneGoal {}

impl Goal for NoneGoal {
    fn activate(&self, cter: &Cter) {
        unimplemented!()
    }

    fn process(&self, cter: &Cter) -> GoalStatus {
        unimplemented!()
    }

    fn terminate(&self) {
        unimplemented!()
    }

    fn get_goal_status(&self) -> GoalStatus {
        unimplemented!()
    }
}

#[derive(Default)]
pub struct AttackGoal {
    status: AtomicCell<GoalStatus>, //目标当前状态
    combin_goal: GoalCombinedTest,  //组合目标
}

impl Goal for AttackGoal {
    fn activate(&self, cter: &Cter) {
        unimplemented!()
    }

    fn process(&self, cter: &Cter) -> GoalStatus {
        unimplemented!()
    }

    fn terminate(&self) {
        unimplemented!()
    }

    fn get_goal_status(&self) -> GoalStatus {
        unimplemented!()
    }
}
