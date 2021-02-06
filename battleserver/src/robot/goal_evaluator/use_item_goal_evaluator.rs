use crate::battle::battle::BattleData;
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::use_item_action::UseItemRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::character::BattleCharacter;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct UseItemGoalEvaluator {
    desirability: AtomicCell<u32>,
}

impl GoalEvaluator for UseItemGoalEvaluator {
    fn calculate_desirability(&self, cter: &BattleCharacter) -> u32 {
        if cter.items.len() > 0 {
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
        let aa = UseItemRobotAction::new(battle_data, sender);
        cter.change_status(Box::new(aa));
    }
}
