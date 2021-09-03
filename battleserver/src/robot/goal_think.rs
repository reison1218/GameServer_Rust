use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::attack_goal_evaluator::AttackTargetGoalEvaluator;
use crate::robot::goal_evaluator::buy_goal_evaluator::BuyGoalEvaluator;
use crate::robot::goal_evaluator::open_cell_goal_evaluator::OpenCellGoalEvaluator;
use crate::robot::goal_evaluator::unlock_goal_evaluator::UnlockGoalEvaluator;
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;
use log::{info, warn};

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
        gt.goal_evaluators
            .push(Box::new(BuyGoalEvaluator::default()));
        gt.goal_evaluators
            .push(Box::new(UnlockGoalEvaluator::default()));
        gt
    }

    ///仲裁goal
    pub fn arbitrate(
        &self,
        robot: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *mut BattleData,
    ) {
        info!("开始执行仲裁");
        if self.goal_evaluators.len() == 0 {
            return;
        }

        //获得仲裁结果
        let best_goal_evaluator = self.goal_evaluators.iter().max_by(|x, y| {
            x.calculate_desirability(robot)
                .cmp(&y.calculate_desirability(robot))
        });

        if best_goal_evaluator.is_none() {
            unsafe {
                let battle_data_mut = battle_data.as_mut().unwrap();
                battle_data_mut.next_turn(true);
                warn!("robot nothing to do!robot_id:{}", robot.get_user_id());
                return;
            }
        }
        let best_goal_evaluator = best_goal_evaluator.unwrap();
        //设置状态
        best_goal_evaluator.set_status(robot, sender, battle_data);
    }
}
