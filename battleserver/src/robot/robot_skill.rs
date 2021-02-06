use crate::battle::battle::BattleData;
use crate::room::map_data::MapCellType;

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
