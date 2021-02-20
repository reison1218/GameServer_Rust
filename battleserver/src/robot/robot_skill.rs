use log::warn;
use rand::Rng;

use crate::{
    battle::battle_enum::skill_type::SKILL_OPEN_NEAR_CELL,
    room::{character::BattleCharacter, map_data::MapCellType},
};
use crate::{
    battle::{battle::BattleData, battle_skill::Skill},
    room::map_data::TileMap,
};

use super::RobotData;

///判断释放条件
pub fn skill_condition(battle_data: &BattleData, skill: &Skill, robot: &RobotData) -> bool {
    let skill_id = skill.id;
    let mut can_use = false;
    if skill.cd_times == 0 {
        can_use = true;
    }
    //特殊使用条件

    true
}

pub fn skill_target(battle_data: &BattleData, skill: &Skill, robot: &RobotData) -> Vec<usize> {
    let skill_id = skill.id;
    let robot_id = robot.robot_id;
    let cter = battle_data.get_battle_cter(Some(robot_id), true).unwrap();
    let mut targets = vec![];
    //匹配技能id进行不同的目标选择
    match skill_id {
        //目标是自己
        i if [211, 313, 321].contains(&i) => {
            targets.push(cter.index_data.map_cell_index.unwrap());
        }
        //除自己外最大血量的目标
        i if [123, 433, 20001, 20002, 20003, 20004, 20005].contains(&i) => {
            let res = get_hp_max_cter(battle_data, robot_id);
            if res.is_none() {
                warn!("get_hp_max_cter res is None!");
                return targets;
            }
            targets.push(res.unwrap());
        }
        _ => {}
    }
    targets
}

///获得除robot_id生命值最高的角色位置
pub fn get_hp_max_cter(battle_data: &BattleData, robot_id: u32) -> Option<usize> {
    let mut res = (0, 0);
    for cter in battle_data.battle_cter.values() {
        //排除死掉的
        if cter.is_died() {
            continue;
        }
        //排除给定robot_id的
        if cter.base_attr.user_id == robot_id {
            continue;
        }
        //对比血量
        if cter.base_attr.hp > res.0 {
            res.0 = cter.base_attr.hp;
            res.1 = cter.index_data.map_cell_index.unwrap();
        }
    }
    //校验返回结果
    if res.1 == 0 {
        return None;
    }
    Some(res.1)
}

///检测是否匹配了
pub fn check_pair(cter: &BattleCharacter) -> bool {
    cter.status.is_pair
}

///检测是否还有位置地图块，有就随机一块出来并返回
pub fn check_unknow_map_cell(tile_map: &TileMap, robot: &RobotData) -> Option<usize> {
    let mut v = vec![];
    for map_cell in tile_map.map_cells.iter() {
        if map_cell.is_world {
            continue;
        }
        let index = map_cell.index;
        for rem_map_cell in robot.remember_map_cell.iter() {
            if rem_map_cell.cell_index == index {
                continue;
            }
            v.push(index);
        }
    }
    if v.len() == 0 {
        return None;
    }
    let rand_index = rand::thread_rng().gen_range(0, v.len());
    let &index = v.get(rand_index).unwrap();
    Some(index)
}

///获得圆形范围aoe范围
pub fn get_roundness_aoe(
    user_id: u32,
    battle_data: BattleData,
    is_check_null: bool,
    is_check_lock: bool,
    is_check_world_cell: bool,
) -> Option<Vec<usize>> {
    let mut res_v = vec![];
    for map_cell in battle_data.tile_map.map_cells.iter() {
        //过滤掉无效地图看
        if map_cell.id <= MapCellType::UnUse.into_u32() {
            continue;
        }
        //检查世界块
        if is_check_world_cell && map_cell.is_world {
            continue;
        }
        //检查是否有锁
        if is_check_lock && map_cell.check_is_locked() {
            continue;
        }
        let mut v = vec![];
        if is_check_null && map_cell.user_id > 0 {
            continue;
        } else if map_cell.user_id == user_id {
            //排除自己
            continue;
        } else if map_cell.user_id > 0 {
            v.push(map_cell.index);
        }

        //以一个地图块为中心点，计算它的周围
        for i in 0..6 {
            let mut coord_index = (map_cell.x, map_cell.y);
            match i {
                0 => {
                    coord_index.0 -= 1;
                    coord_index.1 += 1;
                }
                1 => {
                    coord_index.1 += 1;
                }
                2 => {
                    coord_index.0 += 1;
                }
                3 => {
                    coord_index.0 += 1;
                    coord_index.1 -= 1;
                }
                4 => {
                    coord_index.1 -= 1;
                }
                5 => {
                    coord_index.0 -= 1;
                }
                _ => {}
            }

            let cell_index = battle_data.tile_map.coord_map.get(&coord_index);
            if cell_index.is_none() {
                continue;
            }
            let cell_index = *cell_index.unwrap();
            let cell = battle_data.tile_map.map_cells.get(cell_index);
            if cell.is_none() {
                continue;
            }
            let cell = cell.unwrap();
            if cell.user_id > 0 && cell.user_id != user_id {
                v.push(cell.index);
            }
        }
        res_v.push(v);
    }
    res_v.iter().max().cloned()
}

///获得三角aoe范围
pub fn get_triangle_aoe(user_id: u32, battle_data: &BattleData) -> Option<Vec<usize>> {
    let mut res_v = vec![];
    for map_cell in battle_data.tile_map.map_cells.iter() {
        let mut v = vec![];
        //过滤掉无效地图块
        if map_cell.id <= MapCellType::UnUse.into_u32() {
            continue;
        }
        if map_cell.user_id == user_id {
            continue;
        }
        //把中心点加进去
        if map_cell.user_id > 0 {
            v.push(map_cell.index);
        }
        let mut coord_index = (map_cell.x, map_cell.y);
        //三个方向进行计算
        for i in 0..3 {
            let mut temp_index = coord_index;
            match i {
                0 => {
                    temp_index.0 -= 1;
                    temp_index.1 += 1;
                }
                1 => {
                    temp_index.0 -= 1;
                }
                2 => {
                    temp_index.1 -= 1;
                }
                _ => coord_index = (map_cell.x, map_cell.y),
            }
            let index = battle_data.tile_map.coord_map.get(&temp_index);
            if index.is_none() {
                continue;
            }
            let index = *index.unwrap();
            let res_cell = battle_data.tile_map.map_cells.get(index);
            if res_cell.is_none() {
                continue;
            }
            let res_cell = res_cell.unwrap();
            //排除无效目标
            if res_cell.user_id <= 0 || res_cell.user_id == user_id {
                continue;
            }
            v.push(res_cell.index);
        }
        res_v.push(v);
    }
    res_v.iter().max().cloned()
}

///获得打一排三个的位置
/// 选取规则，除了自己所有人为起点，转一圈，包含人最多，就选中
pub fn get_line_aoe(user_id: u32, battle_data: &BattleData) -> Option<Vec<usize>> {
    let mut v = Vec::new();
    let map_cells = &battle_data.tile_map.map_cells;
    for index in 0..map_cells.len() {
        let cell = map_cells.get(index).unwrap();
        if cell.user_id <= 0 {
            continue;
        }
        if cell.user_id == user_id {
            continue;
        }
        v.push(index);
    }

    let mut res_v = vec![];
    for index in v.iter() {
        let index = *index;
        let map = battle_data.tile_map.map_cells.get(index).unwrap();
        //从六个方向计算
        for i in 0..6 {
            let mut temp_v = vec![];
            //先把起点的人加进去
            temp_v.push(index);
            //每个方向从中心点延伸出去两个格子
            for j in 0..2 {
                let mut coord_index = (map.x, map.y);
                match i {
                    0 => match j {
                        0 => {
                            coord_index.0 -= 1;
                            coord_index.1 += 1;
                        }
                        1 => {
                            coord_index.0 -= 2;
                            coord_index.1 += 2;
                        }
                        _ => {}
                    },
                    1 => match j {
                        0 => {
                            coord_index.1 += 1;
                        }
                        1 => {
                            coord_index.1 += 2;
                        }
                        _ => {}
                    },
                    2 => match j {
                        0 => {
                            coord_index.0 += 1;
                        }
                        1 => {
                            coord_index.0 += 2;
                        }
                        _ => {}
                    },
                    3 => match j {
                        0 => {
                            coord_index.0 += 1;
                            coord_index.0 -= 1;
                        }
                        1 => {
                            coord_index.0 += 2;
                            coord_index.0 -= 2;
                        }
                        _ => {}
                    },
                    4 => match j {
                        0 => {
                            coord_index.0 -= 1;
                        }
                        1 => {
                            coord_index.0 -= 2;
                        }
                        _ => {}
                    },
                    5 => match j {
                        0 => {
                            coord_index.0 += 1;
                        }
                        1 => {
                            coord_index.0 += 2;
                        }
                        _ => {}
                    },
                    _ => {}
                }
                let res = battle_data.tile_map.coord_map.get(&coord_index);
                if res.is_none() {
                    continue;
                }
                let index = *res.unwrap();
                let map_cell = battle_data.tile_map.map_cells.get(index);
                match map_cell {
                    Some(map_cell) => {
                        if map_cell.is_world {
                            continue;
                        }
                        if map_cell.user_id <= 0 || map_cell.user_id == user_id {
                            continue;
                        }
                        temp_v.push(map_cell.index);
                    }
                    None => {}
                }
            }
            res_v.push(temp_v);
        }
    }
    res_v.iter().max().cloned()
}
