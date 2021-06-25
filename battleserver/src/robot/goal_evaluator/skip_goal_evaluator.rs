use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::skip_action::SkipRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct SkipGoalEvaluator {
    // desirability: AtomicCell<u32>,
}

impl GoalEvaluator for SkipGoalEvaluator {
    fn calculate_desirability(&self, battle_player: &BattlePlayer) -> u32 {
        //如果什么都干不了了，则结束turn期望值拉满
        if battle_player.flow_data.residue_movement_points == 0
            && !battle_player.is_can_attack()
            && !battle_player.cter.can_use_skill()
        {
            return 100;
        }
        0
    }

    fn set_status(
        &self,
        robot: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let mut res = SkipRobotAction::new(battle_data, sender);
        res.cter_id = robot.get_cter_id();
        res.robot_id = robot.get_user_id();
        robot.change_robot_status(Box::new(res));
    }
}
