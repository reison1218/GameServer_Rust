use std::collections::VecDeque;

use crossbeam::channel::Sender;

use crate::battle::battle::BattleData;

use super::{
    goal_think::GoalThink, robot_action::RobotStatusAction, robot_task_mgr::RobotTask, RememberCell,
};

pub struct Robot {
    robot_id: u32,
    battle_data: &'static BattleData,
    pub goal_think: GoalThink,                            //机器人think
    pub robot_status: Option<Box<dyn RobotStatusAction>>, //状态,
    pub remember_map_cell: VecDeque<RememberCell>,        //记忆地图块
    pub sender: Sender<RobotTask>,                        //机器人任务sender
}

impl Robot {
    pub fn new(robot_id: u32, battle_data: &'static BattleData, sender: Sender<RobotTask>) -> Self {
        Robot {
            robot_id,
            battle_data,
            goal_think: GoalThink::new(),
            robot_status: None,
            remember_map_cell: VecDeque::new(),
            sender,
        }
    }
}
