use crate::battle::battle::BattleData;
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::AttackRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::character::BattleCharacter;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct AttackTargetGoalEvaluator {
    desirability: AtomicCell<u32>,
}

impl GoalEvaluator for AttackTargetGoalEvaluator {
    fn calculate_desirability(&self, cter: &BattleCharacter) -> u32 {
        if cter.is_can_attack() {
            return 1;
        }
        0
    }

    fn set_status(
        &self,
        cter: &BattleCharacter,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let aa = AttackRobotAction::new(battle_data, sender);
        cter.change_status(Box::new(aa));
    }
}
