pub mod goal_evaluator;
pub mod goal_think;
pub mod robot_action;
pub mod robot_skill;
pub mod robot_status;
pub mod robot_task_mgr;
pub mod robot_trigger;

use crate::battle::{battle::BattleData, battle_player::BattlePlayer};
use crate::robot::goal_think::GoalThink;
use crate::robot::robot_action::RobotStatusAction;
use crate::robot::robot_task_mgr::RobotTask;
use crate::robot::robot_trigger::RobotTriggerType;
use crossbeam::channel::Sender;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use std::collections::VecDeque;

pub const MAX_MEMORY_SIZE: usize = 5;

///回合行为类型
#[derive(Debug, Clone, Copy, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RobotActionType {
    ///无效值
    None,
    ///选择位置
    ChoiceIndex,
    ///普通攻击
    Attack,
    ///使用道具
    UseItem,
    ///跳过turn
    Skip,
    ///翻块
    Open,
    ///使用技能
    Skill,
    ///触发buff
    Buff,
    ///结束展示地图块(解锁玩家状态)
    EndShowMapCell,
    ///结束展示地图块(解锁玩家状态)
    Buy,
}

impl Default for RobotActionType {
    fn default() -> Self {
        RobotActionType::None
    }
}

///记忆地图块结构体
#[derive(Default, Clone)]
pub struct RememberCell {
    pub cell_index: usize, //地图块下标
    pub cell_id: u32,      //地图块id
}

impl RememberCell {
    pub fn new(cell_index: usize, cell_id: u32) -> Self {
        let mut rc = RememberCell::default();
        rc.cell_index = cell_index;
        rc.cell_id = cell_id;
        rc
    }
}

///机器人数据结构体
pub struct RobotData {
    pub robot_id: u32,
    pub battle_data: *mut BattleData,
    pub goal_think: GoalThink,                            //机器人think
    pub robot_status: Option<Box<dyn RobotStatusAction>>, //状态,
    pub remember_map_cell: VecDeque<RememberCell>,        //记忆地图块
    pub sender: Sender<RobotTask>,                        //机器人任务sender
}

impl RobotData {
    ///创建robotdata结构体
    pub fn new(robot_id: u32, battle_data: *mut BattleData, sender: Sender<RobotTask>) -> Self {
        RobotData {
            robot_id,
            battle_data,
            goal_think: GoalThink::new(),
            robot_status: None,
            remember_map_cell: VecDeque::new(),
            sender,
        }
    }

    pub fn can_pair_index(&self) -> Option<usize> {
        unsafe {
            let battle_data = self.battle_data.as_ref().unwrap();
            let robot = battle_data.battle_player.get(&self.robot_id);
            if let None = robot {
                return None;
            }
            let robot = robot.unwrap();
            for &open_index in robot.flow_data.open_map_cell_vec.iter() {
                let res = battle_data.tile_map.map_cells.get(open_index);
                if res.is_none() {
                    continue;
                }
                let match_map_cell = res.unwrap();
                for re_map_cell in self.remember_map_cell.iter() {
                    if match_map_cell.id != re_map_cell.cell_id {
                        continue;
                    }
                    return Some(re_map_cell.cell_index);
                }
            }
            None
        }
    }

    pub fn clone_battle_data_ptr(&self) -> *mut BattleData {
        self.battle_data.clone()
    }

    pub fn get_battle_player_mut_ref(&mut self) -> &mut BattlePlayer {
        unsafe {
            let res = self
                .battle_data
                .as_mut()
                .unwrap()
                .battle_player
                .get_mut(&self.robot_id)
                .unwrap();
            res
        }
    }

    ///思考做做什么，这里会执行仲裁，数值最高的会挑出来进行执行
    pub fn thinking_do_something(&mut self) {
        let robot_id = self.robot_id;
        let sender = self.sender.clone();
        let battle_data_cp = self.clone_battle_data_ptr();
        let self_ptr = self as *mut RobotData;
        unsafe {
            let self_mut = self_ptr.as_mut().unwrap();
            let battle_data = self_mut.battle_data.as_mut().unwrap();
            let robot = battle_data.battle_player.get_mut(&robot_id).unwrap();
            self_mut.goal_think.arbitrate(robot, sender, battle_data_cp);
        }
    }

    pub fn trigger(&mut self, rc: RememberCell, trigger_type: RobotTriggerType) {
        match trigger_type {
            RobotTriggerType::SeeMapCell => {
                self.trigger_see_map_cell(rc);
            }
            RobotTriggerType::MapCellPair => {
                self.trigger_pair_map_cell(rc);
            }
            _ => {
                self.trigger_see_map_cell(rc);
            }
        }
    }
}

impl Clone for RobotData {
    fn clone(&self) -> Self {
        RobotData {
            robot_id: self.robot_id,
            battle_data: self.battle_data.clone(),
            goal_think: self.goal_think.clone(),
            robot_status: None,
            remember_map_cell: self.remember_map_cell.clone(),
            sender: self.sender.clone(),
        }
    }
}
