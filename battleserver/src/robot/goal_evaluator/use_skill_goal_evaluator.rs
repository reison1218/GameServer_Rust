use crate::battle::battle::BattleData;
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_skill::skill_condition;
use crate::robot::robot_status::use_skill_action::UseSkillRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crate::room::character::BattleCharacter;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct UseSkillGoalEvaluator {
    // desirability: AtomicCell<u32>,
}

pub fn get_battle_data_ref(cter: &BattleCharacter) -> &BattleData {
    unsafe {
        cter.robot_data
            .as_ref()
            .unwrap()
            .battle_data
            .as_ref()
            .unwrap()
    }
}

impl GoalEvaluator for UseSkillGoalEvaluator {
    fn calculate_desirability(&self, cter: &BattleCharacter) -> u32 {
        //如果可以使用技能，则直接期望值拉满
        let robot = cter.robot_data.as_ref().unwrap();
        let battle_data = get_battle_data_ref(cter);
        for skill in cter.skills.values() {
            let res = skill_condition(battle_data, skill, robot);
            if res {
                return 100;
            }
        }
        0
    }

    fn set_status(
        &self,
        cter: &BattleCharacter,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let aa = UseSkillRobotAction::new(battle_data, sender);
        cter.change_robot_status(Box::new(aa));
    }
}
