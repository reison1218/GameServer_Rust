use crate::battle::battle::BattleData;
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::choice_index_action::ChoiceIndexRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::character::BattleCharacter;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct ChoiceIndexGoalEvaluator {
    desirability: AtomicCell<u32>,
}

impl GoalEvaluator for ChoiceIndexGoalEvaluator {
    fn calculate_desirability(&self, cter: &BattleCharacter) -> u32 {
        if cter.index_data.map_cell_index.is_none() {
            return 100;
        }
        0
    }

    fn set_status(
        &self,
        cter: &BattleCharacter,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let aa = ChoiceIndexRobotAction::new(battle_data, sender);
        cter.change_status(Box::new(aa));
    }
}
