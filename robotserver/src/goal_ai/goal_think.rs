use crate::goal_ai::attack_target_goal_evaluator::AttackTargetGoalEvaluator;
use crate::goal_ai::cter::Cter;
use crate::goal_ai::goal::Goal;
use crate::goal_ai::goal_combined::GoalCombined;
use crate::goal_ai::goal_evaluator::GoalEvaluator;
use crate::goal_ai::goal_status::GoalStatus;
use crossbeam::atomic::AtomicCell;
use std::collections::VecDeque;

#[derive(Default)]
pub struct GoalThink {
    status: AtomicCell<GoalStatus>,
    attack_bias: AtomicCell<u32>,
    goal_evaluators: Vec<Box<dyn GoalEvaluator>>,
}

impl GoalThink {
    pub fn new() -> Self {
        let mut gt = GoalThink::default();
        let attack = Box::new(AttackTargetGoalEvaluator::new(gt.attack_bias.load()));
        gt.goal_evaluators.push(attack);
        gt
    }
    ///仲裁goal
    pub fn arbitrate(&self, cter: &Cter) {
        let mut best_desirabilty = 0;
        let mut best_index = 0;

        for index in 0..self.goal_evaluators.len() {
            let ge = self.goal_evaluators.get(index).unwrap();
            let desirabilty = ge.calculate_desirability();
            if desirabilty > best_desirabilty {
                best_desirabilty = desirabilty;
                best_index = index;
            }
        }
        let best_goal_evaluator = self.goal_evaluators.get(best_index).unwrap();
        best_goal_evaluator.set_goal(cter);
    }

    pub fn add_sub_goal(&self, combin_goal: Box<dyn Goal>) {
        unimplemented!()
    }

    pub fn add_attack_target(&self) {
        self.remove_all_sub_goals();
    }
}

impl Goal for GoalThink {
    fn activate(&self, cter: &Cter) {
        self.arbitrate(cter);
    }

    fn process(&self, cter: &Cter) -> GoalStatus {
        let sub_goal_status = self.process_sub_goals(cter);
        if sub_goal_status == GoalStatus::Finish || sub_goal_status == GoalStatus::Fail {
            self.status.store(GoalStatus::Idel);
        }
        self.status.load()
    }

    fn terminate(&self) {
        unimplemented!()
    }

    fn get_goal_status(&self) -> GoalStatus {
        self.status.load()
    }
}

impl GoalCombined for GoalThink {
    fn get_sub_goals(&self) -> &mut VecDeque<Box<dyn Goal>> {
        unimplemented!()
    }
}
