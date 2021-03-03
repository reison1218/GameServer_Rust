use crate::battle::battle::BattleData;
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_status::skip_action::SkipRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::character::BattleCharacter;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct SkipGoalEvaluator {
    // desirability: AtomicCell<u32>,
}

impl GoalEvaluator for SkipGoalEvaluator {
    fn calculate_desirability(&self, cter: &BattleCharacter) -> u32 {
        //如果什么都干不了了，则结束turn期望值拉满
        if cter.flow_data.residue_movement_points == 0
            && !cter.is_can_attack()
            && !cter.can_use_skill()
        {
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
        let aa = SkipRobotAction::new(battle_data, sender);
        cter.change_robot_status(Box::new(aa));
    }
}
