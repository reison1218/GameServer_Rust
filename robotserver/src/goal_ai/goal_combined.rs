use crate::goal_ai::cter::Cter;
use crate::goal_ai::goal::Goal;
use crate::goal_ai::goal_status::GoalStatus;
use crossbeam::atomic::AtomicCell;
use crossbeam::queue::ArrayQueue;
use std::collections::VecDeque;

pub trait GoalCombined: Goal {
    fn process_sub_goals(&self, cter: &Cter) -> GoalStatus;

    fn add_sub_goal(&self, goal: Box<dyn Goal>);

    fn remove_all_sub_goals(&self);
}

pub struct GoalCombinedTest {
    sub_goals: AtomicCell<VecDeque<Box<dyn Goal>>>,
}

impl Default for GoalCombinedTest {
    fn default() -> Self {
        let sub_goals: AtomicCell<VecDeque<Box<dyn Goal>>> = AtomicCell::new(VecDeque::new());
        GoalCombinedTest { sub_goals }
    }
}

impl GoalCombined for GoalCombinedTest {
    fn process_sub_goals(&self, cter: &Cter) -> GoalStatus {
        let mut sub_goals = self.sub_goals.take();
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
            self.sub_goals.store(sub_goals);
            return GoalStatus::Finish;
        }

        //取出队列头部目标
        let goal = sub_goals.pop_back().unwrap();
        //推进目标
        let sub_goal_status = goal.process(cter);
        if GoalStatus::Finish == sub_goal_status && sub_goals.len() > 1 {
            self.sub_goals.store(sub_goals);
            return GoalStatus::Active;
        }
        self.sub_goals.store(sub_goals);
        return sub_goal_status;
    }

    fn add_sub_goal(&self, goal: Box<dyn Goal>) {
        let mut sub_goals = self.sub_goals.take();
        sub_goals.push_front(goal);
        self.sub_goals.store(sub_goals);
    }

    fn remove_all_sub_goals(&self) {
        let mut sub_goals = self.sub_goals.take();
        for sg in sub_goals.iter() {
            sg.terminate();
        }
        sub_goals.clear();
        self.sub_goals.store(sub_goals);
    }
}

impl Goal for GoalCombinedTest {
    fn activate(&self, cter: &Cter) {
        unimplemented!()
    }

    fn process(&self, cter: &Cter) -> GoalStatus {
        unimplemented!()
    }

    ///终止
    fn terminate(&self) {
        unimplemented!()
    }

    fn get_goal_status(&self) -> GoalStatus {
        unimplemented!()
    }
}
