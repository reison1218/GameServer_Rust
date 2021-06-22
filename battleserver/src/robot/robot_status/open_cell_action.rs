use std::borrow::Borrow;

use super::*;
use crate::{battle::battle_player::BattlePlayer, robot::RobotActionType};
use log::{error, warn};
use tools::cmd_code::BattleCode;

#[derive(Default)]
pub struct OpenCellRobotAction {
    pub robot_id: u32,
    pub cter_id: u32,
    pub battle_data: Option<*const BattleData>,
    pub status: RobotStatus,
    pub sender: Option<Sender<RobotTask>>,
}

impl OpenCellRobotAction {
    pub fn get_battle_data_ref(&self) -> Option<&BattleData> {
        unsafe {
            if self.battle_data.unwrap().is_null() {
                return None;
            }
            Some(self.battle_data.unwrap().as_ref().unwrap())
        }
    }

    pub fn new(battle_data: *const BattleData, sender: Sender<RobotTask>) -> Self {
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
        info!("robot:{},进入翻地图块状态", self.robot_id);
        self.execute();
    }

    fn execute(&self) {
        let battle_data = self.get_battle_data_ref();
        if battle_data.is_none() {
            warn!("the point *const BattleData is null!");
            return;
        }
        let battle_data = battle_data.unwrap();
        //校验为配对的地图块数量
        if battle_data.tile_map.un_pair_map.is_empty() {
            return;
        }
        let mut v = Vec::new();
        for key in battle_data.tile_map.un_pair_map.keys() {
            v.push(*key);
        }
        let mut rand = rand::thread_rng();
        let mut index = 0;

        let robot_id = self.robot_id;
        let battle_player = battle_data.battle_player.get(&robot_id).unwrap();
        let mut action_type = RobotActionType::Open;

        //剩余翻块次数
        let residue_open_times = battle_player.flow_data.residue_movement_points;

        //剩余次数等于0，则啥也不干，直接返回
        if residue_open_times == 0 {
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
            if let Some(element_index) = element_index {
                index = element_index;
            } else {
                index = rand.gen_range(0..pair_v.len());
                index = *pair_v.get(index).unwrap();
            }
        } else if pair_v.len() == 1 {
            //翻开能够配对的地图块
            index = *pair_v.get(0).unwrap();
        } else {
            let mut is_cd = false;
            for skill in battle_player.cter.skills.values() {
                if skill.cd_times > 0_i8 {
                    is_cd = true;
                    break;
                }
            }
            //如果有技能cd的话就随机在地图里面翻开一个地图块
            if is_cd {
                index = rand.gen_range(0..v.len());
            } else {
                //否则
                let res = rand.gen_range(0..101);
                if res >= 0 && res <= 60 {
                    index = rand.gen_range(0..v.len());
                } else {
                    //跳过turn
                    action_type = RobotActionType::Skip;
                }
            }
        }
        self.send_2_battle(index, action_type, BattleCode::Action);
    }

    fn exit(&self) {
        unimplemented!()
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
}

///计算能够配对的数量
pub fn cal_pair_num(
    battle_data: &BattleData,
    battle_player: &BattlePlayer,
) -> anyhow::Result<(Vec<usize>, Option<usize>)> {
    let mut index_v = Vec::new();
    let mut element_index: Option<usize> = None;

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
    let open_map_cell_vec = &battle_player.flow_data.open_map_cell_vec_history;
    //去掉已经翻开过的,并且添加可以配对的
    for cell_index in open_map_cell_vec.iter() {
        let cell_index = *cell_index;
        let map_cell = battle_data.tile_map.map_cells.get(cell_index).unwrap();
        for re_cell in remember_cells.iter() {
            //如果是已经翻开了的，就跳过
            if cell_index == re_cell.cell_index {
                continue;
            }
            //添加可以配对的
            if map_cell.id == re_cell.cell_id {
                index_v.push(cell_index);
            }
            //添加元素相同的
            if element_index.is_none() && map_cell.element == battle_player.cter.base_attr.element {
                element_index = Some(map_cell.index);
            }
        }
    }
    Ok((index_v, element_index))
}
