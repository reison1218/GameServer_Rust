use std::borrow::Borrow;

use super::*;
use crate::{
    battle::battle_player::BattlePlayer,
    robot::{robot_helper::check_can_open, RobotActionType},
    JsonValue,
};
use log::{error, warn};
use serde_json::Map;
use tools::cmd_code::BattleCode;

#[derive(Default)]
pub struct OpenCellRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub temp_id: u32,
    pub battle_data: Option<*mut BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

impl OpenCellRobotAction {
    pub fn get_battle_data_mut_ref(&self) -> Option<&mut BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }

            Some(self.battle_data.unwrap().as_mut().unwrap())
        }
    }

    pub fn new(battle_data: *mut BattleData, sender: Sender<RobotTask>) -> Self {
        let mut open_cell = OpenCellRobotAction::default();
        open_cell.battle_data = Some(battle_data);
        open_cell.sender = Some(sender);
        open_cell
    }
}

get_mut_ref!(OpenCellRobotAction);

impl RobotStatusAction for OpenCellRobotAction {
    fn set_sender(&mut self, sender: Sender<RobotTask>) {
        self.sender = Some(sender);
    }

    fn get_cter_id(&self) -> u32 {
        self.cter_id
    }

    fn enter(&self) {
        info!("robot:{} 进入翻地图块状态", self.robot_id);
        self.execute();
    }

    fn execute(&self) {
        let battle_data = self.get_battle_data_mut_ref();
        if battle_data.is_none() {
            warn!("the point *const BattleData is null!");
            return;
        }
        let battle_data = battle_data.unwrap();
        //校验为配对的地图块数量
        if battle_data.tile_map.un_pair_map.is_empty() {
            warn!("un_pair_map is empty!");
            return;
        }
        let mut v = Vec::new();
        let robot_id = self.robot_id;
        let battle_player = battle_data.battle_player.get(&robot_id).unwrap();
        for key in battle_data.tile_map.un_pair_map.keys() {
            let map_cell = battle_data.tile_map.map_cells.get(*key).unwrap();
            //跳过自己已翻开的
            if map_cell.open_user == robot_id || map_cell.user_id == robot_id {
                continue;
            }
            //跳过锁住的地图块
            let res = check_can_open(robot_id, map_cell, battle_data);
            if !res {
                continue;
            }
            //判断地图块上面是否有人
            let player = battle_data.battle_player.get(&map_cell.open_user);
            if let Some(player) = player {
                if !player.can_be_move() {
                    continue;
                }
            }
            v.push(*key);
        }
        let mut rand = rand::thread_rng();
        let mut index = None;

        let mut action_type = RobotActionType::Open;

        //剩余翻块次数
        let residue_open_times = battle_player.flow_data.residue_movement_points;

        //剩余次数等于0，则啥也不干，直接返回
        if residue_open_times == 0 {
            warn!("residue_open_times is 0!robot_id:{}", self.robot_id);
            return;
        }
        //计算可以配对多少个
        let res = cal_pair_num(battle_data, battle_player);
        if let Err(e) = res {
            error!("{:?}", e);
            return;
        }
        let (pair_v, element_index) = res.unwrap();

        //大于1个时，优先配对与自己元素相同的地图块
        if pair_v.len() > 1 {
            if element_index.len() > 1 {
                let rand_index = rand.gen_range(0..element_index.len());
                index = Some(*element_index.get(rand_index).unwrap());
            } else {
                let rand_index = rand.gen_range(0..pair_v.len());
                index = Some(*pair_v.get(rand_index).unwrap());
            }
        } else if pair_v.len() == 1 {
            //翻开能够配对的地图块
            index = Some(*pair_v.get(0).unwrap());
        } else {
            //如果没有，则随机翻开一个未知地图块
            let mut user_id;
            let mut map_cell;
            let robot_data = battle_player.robot_data.as_ref().unwrap();
            let mut unknown_v = vec![];
            'out: for (&map_cell_index, _) in battle_data.tile_map.un_pair_map.iter() {
                map_cell = battle_data.tile_map.map_cells.get(map_cell_index).unwrap();
                user_id = map_cell.user_id;
                if user_id > 0 {
                    let player = battle_data.battle_player.get(&user_id).unwrap();
                    if !player.is_can_attack() {
                        continue;
                    }
                }

                if map_cell.is_world() {
                    continue;
                }

                if map_cell.is_market() {
                    continue;
                }

                if map_cell.check_is_locked() {
                    continue;
                }

                if !robot_data.remember_map_cell.is_empty() {
                    for re_cell in robot_data.remember_map_cell.iter() {
                        if re_cell.cell_index == map_cell_index {
                            continue 'out;
                        }
                        unknown_v.push(map_cell_index);
                    }
                } else {
                    unknown_v.push(map_cell_index);
                }
            }
            if unknown_v.len() > 0 {
                let rand_index = rand.gen_range(0..unknown_v.len());
                index = Some(*unknown_v.get(rand_index).unwrap());
            }
        }
        let indes_res;
        match index {
            Some(index) => {
                indes_res = index;
                info!("选中index:{}", indes_res);
            }
            None => {
                indes_res = 0;
                action_type = RobotActionType::Skip;
                info!("没选中，执行跳过");
            }
        }
        self.send_2_battle(indes_res, action_type, BattleCode::Action);
    }

    fn exit(&self) {
        // info!("robot:{} 退出打开地块状态！", self.robot_id);
    }

    fn get_status(&self) -> RobotStatus {
        self.status
    }

    fn get_robot_id(&self) -> u32 {
        self.robot_id
    }

    fn get_sender(&self) -> &Sender<RobotTask> {
        self.sender.as_ref().unwrap()
    }

    fn send_2_battle(
        &self,
        target_index: usize,
        robot_action_type: RobotActionType,
        cmd: BattleCode,
    ) {
        let mut robot_task = RobotTask::default();
        robot_task.action_type = robot_action_type;
        robot_task.robot_id = self.robot_id;
        let mut map = Map::new();
        map.insert("value".to_owned(), JsonValue::from(target_index));
        map.insert("cmd".to_owned(), JsonValue::from(cmd.into_u32()));
        robot_task.data = JsonValue::from(map);
        let res = self.get_sender().send(robot_task);
        if let Err(e) = res {
            error!("{:?}", e);
        }
    }
}

///计算能够配对的数量
pub fn cal_pair_num(
    battle_data: &BattleData,
    battle_player: &BattlePlayer,
) -> anyhow::Result<(Vec<usize>, Vec<usize>)> {
    let mut index_v = Vec::new();
    let mut element_v = Vec::new();
    let robot_id = battle_player.get_user_id();
    //拿到机器人数据
    let robot_data = battle_player.get_robot_data_ref();
    if let Err(e) = robot_data {
        error!("{:?}", e);
        anyhow::bail!(e)
    }
    let robot_data = robot_data.unwrap();

    //机器人记忆的地图块
    let remember_cells = robot_data.remember_map_cell.borrow();
    //这个turn放开的地图块下标
    let element = battle_player.cter.base_attr.element;
    let mut cell_index;
    for cell in remember_cells.iter() {
        cell_index = cell.cell_index;
        let map_cell = battle_data.tile_map.map_cells.get(cell_index).unwrap();
        //去掉已经翻开过的
        if map_cell.open_user > 0 {
            continue;
        }
        let res = check_can_open(robot_id, map_cell, battle_data);
        if !res {
            continue;
        }
        for re_cell in remember_cells.iter() {
            //去掉自己
            if re_cell.cell_index == cell_index {
                continue;
            }
            //添加可以配对的
            if map_cell.id == re_cell.cell_id {
                index_v.push(cell_index);
                //添加元素相同的
                if map_cell.element == element {
                    element_v.push(map_cell.index);
                    break;
                }
            }
        }
    }
    if index_v.len() > 1 {
        info!("机器人找出配对:{:?},robot:{}", index_v, robot_id);
    }
    Ok((index_v, element_v))
}
