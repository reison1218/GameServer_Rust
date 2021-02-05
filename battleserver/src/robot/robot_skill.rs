use crate::battle::battle::BattleData;
use crate::room::map_data::MapCellType;
use std::borrow::Borrow;

///获得打一排三个的位置
/// 选取规则，除了自己所有人为起点，转一圈，包含人最多，就选中
pub fn get_line_aoe(user_id: u32, battle_data: &BattleData) -> Vec<usize> {
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

        let mut up_index = map.index;
        let mut right_up_index = map.index;
        let mut left_up_index = map.index;
        let mut down_index = map.index;
        let mut right_down_index = map.index;
        let mut left_down_index = map.index;
        let mut up_v = vec![];
        let mut right_up_v = vec![];
        let mut left_up_v = vec![];
        let mut down_v = vec![];
        let mut right_down_v = vec![];
        let mut left_down_v = vec![];
        for _ in 0..2 {
            //向上
            up_index += 5;
            let res = battle_data.tile_map.map_cells.get(up_index);
            match res {
                Some(map_cell) => {
                    if map_cell.user_id > 0 {
                        up_v.push(map_cell.index);
                    }
                }
                None => {}
            }
            //右上
            right_up_index += 6;
            let res = battle_data.tile_map.map_cells.get(right_up_index);
            match res {
                Some(map_cell) => {
                    if map_cell.user_id > 0 {
                        right_up_v.push(map_cell.index);
                    }
                }
                None => {}
            }
            //右下
            right_down_index += 1;
            let res = battle_data.tile_map.map_cells.get(right_down_index);
            match res {
                Some(map_cell) => {
                    if map_cell.user_id > 0 {
                        right_down_v.push(map_cell.index);
                    }
                }
                None => {}
            }
            //下方
            down_index -= 5;
            let res = battle_data.tile_map.map_cells.get(down_index);
            match res {
                Some(map_cell) => {
                    if map_cell.user_id > 0 {
                        down_v.push(map_cell.index);
                    }
                }
                None => {}
            }
            //左下
            left_down_index -= 6;
            let res = battle_data.tile_map.map_cells.get(left_down_index);
            match res {
                Some(map_cell) => {
                    if map_cell.user_id > 0 {
                        left_down_v.push(map_cell.index);
                    }
                }
                None => {}
            }
            //左上
            left_up_index -= 1;
            let res = battle_data.tile_map.map_cells.get(left_up_index);
            match res {
                Some(map_cell) => {
                    if map_cell.user_id > 0 {
                        left_up_v.push(map_cell.index);
                    }
                }
                None => {}
            }
        }
        res_v.push(up_v.clone());
        res_v.push(right_up_v.clone());
        res_v.push(left_up_v.clone());
        res_v.push(down_v.clone());
        res_v.push(right_down_v.clone());
        res_v.push(left_down_v.clone());
    }
    let res = res_v.iter().max();
    match res {
        Some(res) => res.clone(),
        None => {
            vec![]
        }
    }
}
