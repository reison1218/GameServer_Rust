use crate::battle::battle::BattleData;
use crate::robot::goal_evaluator::attack_goal_evaluator::AttackTargetGoalEvaluator;
use crate::robot::goal_evaluator::open_cell_goal_evaluator::OpenCellGoalEvaluator;
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::character::BattleCharacter;
use crossbeam::channel::Sender;

use super::goal_evaluator::choice_index_goal_evaluator::ChoiceIndexGoalEvaluator;
use crate::robot::goal_evaluator::skip_goal_evaluator::SkipGoalEvaluator;
use crate::robot::goal_evaluator::use_skill_goal_evaluator::UseSkillGoalEvaluator;

#[derive(Default)]
pub struct GoalThink {
    goal_evaluators: Vec<Box<dyn GoalEvaluator>>,
}

impl Clone for GoalThink {
    fn clone(&self) -> Self {
        GoalThink::new()
    }
}

impl GoalThink {
    pub fn new() -> Self {
        let mut gt = GoalThink::default();
        gt.goal_evaluators
            .push(Box::new(AttackTargetGoalEvaluator::default()));
        gt.goal_evaluators
            .push(Box::new(OpenCellGoalEvaluator::default()));
        gt.goal_evaluators
            .push(Box::new(ChoiceIndexGoalEvaluator::default()));
        gt.goal_evaluators
            .push(Box::new(SkipGoalEvaluator::default()));
        gt.goal_evaluators
            .push(Box::new(UseSkillGoalEvaluator::default()));
        gt
    }

    ///仲裁goal
    pub fn arbitrate(
        &self,
        cter: &BattleCharacter,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        println!("开始执行仲裁");
        let mut best_desirabilty = 0;
        let mut best_index = 0;
        if self.goal_evaluators.len() == 0 {
            return;
        }
        for index in 0..self.goal_evaluators.len() {
            let ge = self.goal_evaluators.get(index).unwrap();
            let desirabilty = ge.calculate_desirability(cter);
            if desirabilty > best_desirabilty {
                best_desirabilty = desirabilty;
                best_index = index;
            }
        }

        let best_goal_evaluator = self.goal_evaluators.get(best_index).unwrap();
        best_goal_evaluator.set_status(cter, sender, battle_data);
    }
}
