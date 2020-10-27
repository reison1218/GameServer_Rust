use crate::goal_ai::cter::Cter;
use crate::goal_ai::goal_status::GoalStatus;
use crate::goal_ai::goals::goal::Goal;
use crate::goal_ai::goals::goal_combined::GoalCombined;
use crossbeam::atomic::AtomicCell;
use std::borrow::BorrowMut;
use std::collections::VecDeque;
use tools::macros::GetMutRef;

#[derive(Default)]
pub struct GoalCalAttackTarget {
    pub status: AtomicCell<GoalStatus>,
    pub sub_goals: VecDeque<Box<dyn Goal>>,
}
tools::get_mut_ref!(GoalCalAttackTarget);

impl Goal for GoalCalAttackTarget {
    fn activate(&self, cter: &Cter) {
        println!("激活GoalCalAttackTarget目标");
        self.status.swap(GoalStatus::Active);
        self.remove_all_sub_goals();

        //添加其他子目标
        // self.add_sub_goal()
    }

    fn process(&self, cter: &Cter) -> GoalStatus {
        println!("执行GoalCalAttackTarget");
        let status = self.process_sub_goals(cter);
        return status;
    }

    fn terminate(&self) {
        self.status.store(GoalStatus::Finish);
    }

    fn get_goal_status(&self) -> GoalStatus {
        self.status.load()
    }
}

impl GoalCombined for GoalCalAttackTarget {
    fn get_sub_goals(&self) -> &mut VecDeque<Box<dyn Goal>> {
        self.get_mut_ref().sub_goals.borrow_mut()
    }
}
