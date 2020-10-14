use crate::goal_ai::goal::{CombinedGoal, Goal, NoneGoal};
use crate::goal_ai::goal_status::GoalStatus;
use crossbeam::atomic::AtomicCell;

pub struct TargetingSystem {
    owner_cter: &'static Cter,
    current_target: &'static Cter,
}

impl TargetingSystem {
    pub fn update(&self) {}

    pub fn get_target(&self) -> &'static Cter {
        self.current_target
    }
}

pub struct Cter {
    pub target_system: TargetingSystem,
    pub goal_think: GoalThink,
    pub id: AtomicCell<u32>, //角色唯一id
    pub goal: Box<dyn Goal>, //目标
}
impl Cter {
    pub fn udpate(&self) {}
}

impl Default for Box<dyn Goal> {
    fn default() -> Self {
        Box::new(NoneGoal::default())
    }
}

pub struct GoalThink {
    goal_evaluators: Vec<Box<dyn GoalEvaluator>>,
}

impl GoalThink {
    pub fn arbitrate(&self) {}

    pub fn open_cell(&self) {}

    pub fn attack(&self) {}
}

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

///评估trait
pub trait GoalEvaluator {
    ///计算期望值
    fn calculate_desirability(&self) -> u32;

    ///设置评估
    fn set_goal(cter: &Cter);
}

///测试评估结构体
pub struct GoalTestEvaluator {
    pub m_d_cter_bias: u32,
}

impl GoalEvaluator for GoalTestEvaluator {
    fn calculate_desirability(&self) -> u32 {
        unimplemented!()
    }

    fn set_goal(cter: &Cter) {
        unimplemented!()
    }
}
