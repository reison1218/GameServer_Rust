use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::use_item_action::UseItemRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct UseItemGoalEvaluator {
    // desirability: AtomicCell<u32>,
}

impl GoalEvaluator for UseItemGoalEvaluator {
    fn calculate_desirability(&self, battle_player: &BattlePlayer) -> u32 {
        if battle_player.cter.items.len() > 0 {
            return 0;
        }
        0
    }

    fn set_status(
        &self,
        robot: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let mut res = UseItemRobotAction::new(battle_data, sender);
        res.cter_id = robot.get_cter_id();
        res.robot_id = robot.get_user_id();
        robot.change_robot_status(Box::new(res));
    }
}
