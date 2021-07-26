use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_evaluator::GoalEvaluator;
use crate::robot::robot_skill::{skill_condition, skill_target};
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
        if !battle_player.get_current_cter().map_cell_index_is_choiced() {
            return 0;
        }
        //如果可以使用技能，则直接期望值拉满
        let robot = battle_player.robot_data.as_ref().unwrap();
        let battle_data = get_battle_data_ref(battle_player);
        for skill in battle_player.get_current_cter().skills.values() {
            //变身技能先跳过
            let res = skill_condition(battle_data, skill, robot);
            if !res {
                continue;
            }
            let targets = skill_target(battle_data, skill, robot);
            if let Err(_) = targets {
                continue;
            }
            return 100;
        }
        0
    }

    fn set_status(
        &self,
        robot: &BattlePlayer,
        sender: Sender<RobotTask>,
        battle_data: *mut BattleData,
    ) {
        let mut res = UseSkillRobotAction::new(battle_data, sender);
        res.cter_id = robot.get_cter_temp_id();
        res.robot_id = robot.get_user_id();
        res.temp_id = robot.robot_data.as_ref().unwrap().temp_id;
        robot.change_robot_status(Box::new(res));
    }
}
