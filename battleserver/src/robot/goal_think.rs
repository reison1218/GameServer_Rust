use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::attack_goal_evaluator::AttackTargetGoalEvaluator;
use crate::robot::goal_evaluator::open_cell_goal_evaluator::OpenCellGoalEvaluator;
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;
use log::info;

use super::goal_evaluator::choice_index_goal_evaluator::ChoiceIndexGoalEvaluator;
use crate::robot::goal_evaluator::skip_goal_evaluator::SkipGoalEvaluator;
use crate::robot::goal_evaluator::use_item_goal_evaluator::UseItemGoalEvaluator;
use crate::robot::goal_evaluator::use_skill_goal_evaluator::UseSkillGoalEvaluator;

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
        let mut gt = GoalThink {
            goal_evaluators: vec![],
        };
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
        gt.goal_evaluators
            .push(Box::new(UseItemGoalEvaluator::default()));
        gt
    }

    ///仲裁goal
    pub fn arbitrate(
        &mut self,
        robot: &mut BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        info!("开始执行仲裁");
        let mut best_desirabilty = 0;
        let mut best_index = 0;
        if self.goal_evaluators.len() == 0 {
            return;
        }
        //开始执行仲裁
        for index in 0..self.goal_evaluators.len() {
            let ge = self.goal_evaluators.get(index).unwrap();
            let desirabilty = ge.calculate_desirability(robot);
            if desirabilty > best_desirabilty {
                best_desirabilty = desirabilty;
                best_index = index;
            }
        }
        //获得仲裁结果
        let best_goal_evaluator = self.goal_evaluators.get(best_index).unwrap();
        //设置状态
        best_goal_evaluator.set_status(robot, sender, battle_data);
    }
}
