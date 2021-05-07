use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_skill::skill_condition;
use crate::robot::robot_status::use_skill_action::UseSkillRobotAction;
use crate::robot::robot_task_mgr::RobotTask;
use crossbeam::channel::Sender;

#[derive(Default)]
pub struct UseSkillGoalEvaluator {
    // desirability: AtomicCell<u32>,
}

pub fn get_battle_data_ref(battle_player: &BattlePlayer) -> &BattleData {
    unsafe {
        battle_player
            .robot_data
            .as_ref()
            .unwrap()
            .battle_data
            .as_ref()
            .unwrap()
    }
}

impl GoalEvaluator for UseSkillGoalEvaluator {
    fn calculate_desirability(&self, battle_player: &BattlePlayer) -> u32 {
        //如果可以使用技能，则直接期望值拉满
        let robot = battle_player.robot_data.as_ref().unwrap();
        let battle_data = get_battle_data_ref(battle_player);
        for skill in battle_player.cter.skills.values() {
            let res = skill_condition(battle_data, skill, robot);
            if res {
                return 100;
            }
        }
        0
    }

    fn set_status(
        &self,
        battle_player: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *const BattleData,
    ) {
        let aa = UseSkillRobotAction::new(battle_data, sender);
        battle_player.change_robot_status(Box::new(aa));
    }
}
