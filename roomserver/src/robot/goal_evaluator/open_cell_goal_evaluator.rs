use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::OpenCell;
use crate::room::character::BattleCharacter;
use crossbeam::atomic::AtomicCell;

#[derive(Default)]
pub struct OpenCellGoalEvaluator {
    desirability: AtomicCell<u32>,
}

impl GoalEvaluator for OpenCellGoalEvaluator {
    fn calculate_desirability(&self, cter: &BattleCharacter) -> u32 {
        if cter.flow_data.residue_open_times > 0 {
            return 1;
        }
        0
    }

    fn set_status(&self, cter: &BattleCharacter) {
        cter.change_status(Box::new(OpenCell::default()));
    }
}
