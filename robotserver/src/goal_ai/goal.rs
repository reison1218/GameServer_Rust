use crate::goal_ai::cter::Cter;
use crate::goal_ai::goal_status::GoalStatus;
use crossbeam::atomic::AtomicCell;
use crossbeam::queue::ArrayQueue;
use std::borrow::Borrow;

///目标trait
pub trait Goal: Send + 'static {
    ///设置目标下标
    fn set_target_index(&self, target_index: u32);

    ///计算期望值
    fn calculate_expect(&self, cter: &Cter) -> u32;

    ///激活
    fn activate(&self, cter: &Cter);

    ///推进
    fn process(&self, cter: &Cter) -> GoalStatus;

    ///终止
    fn terminate(&self, cter: &Cter);

    ///添加目标
    fn add_goal(&self, goal: Box<dyn Goal>);

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
}

#[derive(Default)]
pub struct NoneGoal {}

impl Goal for NoneGoal {
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

pub struct CombinedGoal {
    goals: ArrayQueue<Box<dyn Goal>>,
}

impl Default for CombinedGoal {
    fn default() -> Self {
        let goals: ArrayQueue<Box<dyn Goal>> = ArrayQueue::new(128);
        CombinedGoal { goals }
    }
}

impl CombinedGoal {
    pub fn process_sub_goal(&self, cter: &Cter) {
        if self.goals.is_empty() {
            return;
        }
        loop {
            let goal = self.goals.pop();
            if let None = goal {
                continue;
            }
            let goal = goal.unwrap();
            //激活这个目标
            goal.activate(cter);
            //推进这个目标
            let status_res = goal.process(cter);
            if status_res == GoalStatus::Finish && status_res == GoalStatus::Fail {
                goal.terminate(cter);
            }
        }
    }

    pub fn remove_all_goals(&self, cter: &Cter) {
        loop {
            let goal = self.goals.pop();
            if let None = goal {
                continue;
            }
            goal.unwrap().terminate(cter);
        }
    }
}

impl Goal for CombinedGoal {
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

    ///终止
    fn terminate(&self, cter: &Cter) {
        unimplemented!()
    }

    fn add_goal(&self, goal: Box<dyn Goal>) {
        self.goals.push(goal);
    }

    fn get_goal_status(&self) -> GoalStatus {
        unimplemented!()
    }
}

#[derive(Default)]
pub struct CellGoal {
    index: AtomicCell<u32>,         //下标
    user_id: AtomicCell<u32>,       //玩家id
    status: AtomicCell<GoalStatus>, //目标当前状态
    combin_goal: CombinedGoal,      //组合目标
}

impl Goal for CellGoal {
    fn set_target_index(&self, target_index: u32) {
        unimplemented!()
    }

    fn calculate_expect(&self, cter: &Cter) -> u32 {
        unimplemented!()
    }

    fn activate(&self, cter: &Cter) {
        self.status.swap(GoalStatus::Active);
    }

    fn process(&self, cter: &Cter) -> GoalStatus {
        let mut user_id = self.user_id.take();
        user_id = cter.id.take();
        self.user_id.store(user_id);
        self.status.swap(GoalStatus::Finish);
        self.status.take()
    }

    fn terminate(&self, cter: &Cter) {
        let mut res = cter.goal.as_ref();
        if self.status.take() == GoalStatus::Finish {
            res.terminate(cter);
        }
    }

    fn add_goal(&self, goal: Box<dyn Goal>) {
        unimplemented!()
    }

    fn get_goal_status(&self) -> GoalStatus {
        self.status.take()
    }
}

#[derive(Default)]
pub struct Attack {
    status: GoalStatus,
}

impl Goal for Attack {
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
