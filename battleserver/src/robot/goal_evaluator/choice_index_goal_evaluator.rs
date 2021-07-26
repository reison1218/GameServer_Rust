use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::choice_index_action::ChoiceIndexRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct ChoiceIndexGoalEvaluator {
    // desirability: AtomicCell<u32>,
}

impl GoalEvaluator for ChoiceIndexGoalEvaluator {
    fn calculate_desirability(&self, battle_player: &BattlePlayer) -> u32 {
        //如果没有选择站位，则期望值拉满
        if battle_player
            .get_current_cter()
            .index_data
            .map_cell_index
            .is_none()
        {
            return 100;
        }
        0
    }

    fn set_status(
        &self,
        robot: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *mut BattleData,
    ) {
        let mut res = ChoiceIndexRobotAction::new(battle_data, sender);
        res.cter_id = robot.get_cter_temp_id();
        res.robot_id = robot.get_user_id();
        res.temp_id = robot.robot_data.as_ref().unwrap().temp_id;
        robot.change_robot_status(Box::new(res));
    }
}
