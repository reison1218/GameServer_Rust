use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::unlock_action::UnlockRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct UnlockGoalEvaluator {
    //desirability: AtomicCell<u32>,
}

impl GoalEvaluator for UnlockGoalEvaluator {
    fn calculate_desirability(&self, robot: &BattlePlayer) -> u32 {
        if !robot.cter.map_cell_index_is_choiced() {
            return 0;
        }
        //如果状态是可以攻击，期望值大于0，当没有其他高优先级的事件，则执行攻击
        if robot.is_locked() {
            return 200;
        }
        0
    }

    fn set_status(
        &self,
        robot: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *mut BattleData,
    ) {
        let mut res = UnlockRobotAction::new(battle_data, sender);
        res.cter_id = robot.get_cter_id();
        res.robot_id = robot.get_user_id();
        res.temp_id = robot.robot_data.as_ref().unwrap().temp_id;
        robot.change_robot_status(Box::new(res));
    }
}
