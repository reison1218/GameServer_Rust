use std::time::Duration;

use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::attack_action::AttackRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct AttackTargetGoalEvaluator {
    //desirability: AtomicCell<u32>,
}

impl GoalEvaluator for AttackTargetGoalEvaluator {
    fn calculate_desirability(&self, robot: &BattlePlayer) -> u32 {
        std::thread::sleep(Duration::from_secs(2));
        //如果状态是可以攻击，期望值大于0，当没有其他高优先级的事件，则执行攻击
        if robot.is_can_attack() {
            return 80;
        }
        0
    }

    fn set_status(
        &self,
        battle_player: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let aa = AttackRobotAction::new(battle_data, sender);
        battle_player.change_robot_status(Box::new(aa));
    }
}
