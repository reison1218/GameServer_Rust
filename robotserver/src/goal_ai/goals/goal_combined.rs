use crate::goal_ai::cter::Cter;
use crate::goal_ai::goal_status::GoalStatus;
use crate::goal_ai::goals::goal::Goal;
use crossbeam::atomic::AtomicCell;
use crossbeam::queue::ArrayQueue;
use std::collections::VecDeque;

///组合目标trait
pub trait GoalCombined: Goal {
    fn get_sub_goals(&self) -> &mut VecDeque<Box<dyn Goal>>;

    fn process_sub_goals(&self, cter: &Cter) -> GoalStatus {
        let mut sub_goals = self.get_sub_goals();
        if sub_goals.is_empty() {
            return GoalStatus::Idel;
        }
        loop {
            let goal = sub_goals.front().unwrap();
            //如果成功了，或者失败了，就终止，并将目标从队列弹出
            if goal.is_finished() || goal.is_failed() {
                goal.terminate();
                sub_goals.pop_front();
            }
        }

        //如果子目标队列是空到直接return
        if sub_goals.is_empty() {
            return GoalStatus::Finish;
        }

        //取出队列头部目标
        let goal = sub_goals.pop_back().unwrap();
        //推进目标
        let sub_goal_status = goal.process(cter);
        if GoalStatus::Finish == sub_goal_status && sub_goals.len() > 1 {
            return GoalStatus::Active;
        }
        return sub_goal_status;
    }

    fn add_sub_goal(&self, goal: Box<dyn Goal>) {
        let mut sub_goals = self.get_sub_goals();
        sub_goals.push_front(goal);
    }

    fn remove_all_sub_goals(&self) {
        let mut sub_goals = self.get_sub_goals();
        for sg in sub_goals.iter() {
            sg.terminate();
        }
        sub_goals.clear();
    }
}
