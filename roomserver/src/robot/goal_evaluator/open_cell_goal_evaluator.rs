use crate::battle::battle::BattleData;
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::open_cell_action::OpenCellRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::character::BattleCharacter;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Sender;

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

    fn set_status(
        &self,
        cter: &BattleCharacter,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let oa = OpenCellRobotAction::new(battle_data, sender);
        cter.change_status(Box::new(oa));
    }
}
