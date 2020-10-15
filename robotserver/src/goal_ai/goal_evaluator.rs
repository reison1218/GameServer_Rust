use crate::goal_ai::cter::Cter;
use crate::goal_ai::goal::Goal;

///评估trait
pub trait GoalEvaluator: Send + 'static {
    ///计算期望值
    fn calculate_desirability(&self) -> u32;

    ///设置评估
    fn set_goal(&self, cter: &Cter);
}

///测试评估结构体
pub struct GoalTestEvaluator {
    pub m_d_cter_bias: u32,
}

impl GoalEvaluator for GoalTestEvaluator {
    fn calculate_desirability(&self) -> u32 {
        unimplemented!()
    }

    fn set_goal(&self, cter: &Cter) {
        cter.goal_think.add_attack_target();
    }
}
