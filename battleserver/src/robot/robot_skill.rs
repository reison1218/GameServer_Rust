use log::{error, warn};
use rand::Rng;
use serde_json::{Map, Value};
use tools::cmd_code::BattleCode;

use crate::battle::{battle_enum::TargetType, battle_player::BattlePlayer};
use crate::handlers::battle_handler::check_skill_useable;
use crate::room::map_data::MapCellType;
use crate::{
    battle::{battle::BattleData, battle_skill::Skill},
    room::map_data::TileMap,
};

use super::goal_evaluator::use_skill_goal_evaluator::get_battle_data_ref;
use super::robot_task_mgr::RobotTask;
use super::{RobotActionType, RobotData};

///机器人使用技能
pub fn robot_use_skill(battle_data: &BattleData, skill: &Skill, robot: &RobotData) -> bool {
    let robot_id = robot.robot_id;
    //先判断技能释放条件
    let res = skill_condition(battle_data, skill, robot);
    //可以释放就往下走
    if !res {
        return false;
    }
    //获取技能释放目标
    let targets = skill_target(battle_data, skill, robot);
    if targets.is_err() {
        return false;
    }
    let targets = targets.unwrap();
    let skill_id = skill.id;
    //创建机器人任务执行
    let mut robot_task = RobotTask::default();
    robot_task.action_type = RobotActionType::Skill;
    let mut map = Map::new();
    map.insert("user_id".to_owned(), Value::from(robot_id));
    map.insert("target_index".to_owned(), Value::from(targets));
    map.insert("cmd".to_owned(), Value::from(BattleCode::Action.into_u32()));
    map.insert("skill_id".to_owned(), Value::from(skill_id));
    robot_task.data = Value::from(map);
    let res = robot.sender.send(robot_task);
    if let Err(e) = res {
        error!("{:?}", e);
    }
    true
}

///判断释放条件
pub fn skill_condition(battle_data: &BattleData, skill: &Skill, robot: &RobotData) -> bool {
    let skill_id = skill.id;
    let mut can_use = false;
    let robot_id = robot.robot_id;
    let skill_judge = skill.skill_temp.skill_judge as u32;
    //如果cd好了就设置状态
    if skill.cd_times == 0 {
        can_use = true;
    }

    let battle_player = battle_data.battle_player.get(&robot_id).unwrap();

    let res = check_skill_useable(battle_player, skill);
    if let Err(_) = res {
        return false;
    }

    let res = battle_data.check_skill_judge(robot_id, skill_judge, Some(skill_id), None);
    if let Err(_) = res {
        return false;
    }
    //特殊使用条件
    match skill_id {
        //判断是否有未知地图快
        i if [113].contains(&i) => {
            can_use = check_unknow_map_cell(&battle_data.tile_map, robot).is_some();
        }
        //判断是否配对
        i if [211].contains(&i) => {
            let battle_player = battle_data.battle_player.get(&robot_id).unwrap();
            can_use = check_pair(battle_player);
        }
        //判断有没有地图块可以翻
        i if [223].contains(&i) => {
            let targets = skill_target(battle_data, skill, robot);
            if let Err(_) = targets {
                can_use = false;
            } else {
                can_use = true;
            }
        }
        //判断周围有没有人
        i if [313].contains(&i) => {
            can_use = no_near_user(battle_data, robot_id);
        }
        //选中至少2个目标
        i if [411].contains(&i) => {
            let res = get_line_aoe(robot_id, battle_data);
            match res {
                Some(v) => {
                    if v.len() > 1 {
                        can_use = true;
                    }
                }
                None => {
                    can_use = false;
                }
            }
        }
        _ => {}
    }
    can_use
}

///选取技能目标
pub fn skill_target(
    battle_data: &BattleData,
    skill: &Skill,
    robot: &RobotData,
) -> anyhow::Result<Vec<usize>> {
    let skill_id = skill.id;
    let robot_id = robot.robot_id;
    let mut targets = vec![];
    //匹配技能id进行不同的目标选择
    match skill_id {
        //目标是自己
        i if [211, 313, 321].contains(&i) => {
            // targets.push(battle_player.cter.index_data.map_cell_index.unwrap());
        }
        //除自己外最大血量的目标
        i if [123, 433, 20001, 20002, 20003, 20004, 20005].contains(&i) => {
            let res = get_hp_max_cter(battle_data, robot_id);
            if res.is_none() {
                warn!("get_hp_max_cter res is None!")
            }
            targets.push(res.unwrap());
        }
        //随机未知地图块
        i if [113].contains(&i) => {
            let res = check_unknow_map_cell(&battle_data.tile_map, robot);
            if let Some(index) = res {
                targets.push(index);
            }
        }
        //获得记忆队列中的地图块
        i if [223].contains(&i) => {
            let res = skill_open_near_cell_robot(battle_data, robot);
            if res.is_none() {
                warn!("")
            }
            targets.push(res.unwrap());
        }
        //直线三个aoe
        i if [411].contains(&i) => {
            let res = get_line_aoe(robot_id, battle_data);
            match res {
                Some(res) => {
                    targets.extend_from_slice(res.as_slice());
                }
                None => {
                    warn!("get_triangle_aoe could not find any target!")
                }
            }
        }
        //随机不在记忆队列中的地图块
        i if [423].contains(&i) => {
            let res = rand_not_remember_map_cell(&battle_data.tile_map, robot);
            targets.push(res);
        }
        //变身技能，计算⭕️
        i if [431].contains(&i) => {
            let res = get_roundness_aoe(robot_id, battle_data, true, false, true, true);
            match res {
                Some(res) => {
                    targets.extend_from_slice(res.as_slice());
                }
                None => {
                    warn!("get_roundness_aoe could not find any target!")
                }
            }
        }
        //⭕️aoe，包括中心，人数越多越好
        i if [432].contains(&i) => {
            let res = get_roundness_aoe(robot_id, battle_data, false, false, false, false);
            match res {
                Some(res) => {
                    targets.extend_from_slice(res.as_slice());
                }
                None => {
                    warn!("get_roundness_aoe could not find any target!")
                }
            }
        }
        _ => {}
    }
    Ok(targets)
}

///随机一个不在记忆队列中的地图块
pub fn rand_not_remember_map_cell(tile_map: &TileMap, robot: &RobotData) -> usize {
    let remember_map_cell = &robot.remember_map_cell;

    let mut not_c_v = vec![];
    let mut v = vec![];

    for map_cell in tile_map.map_cells.iter() {
        //过滤世界块
        if map_cell.is_world() {
            continue;
        }
        //过滤无效块
        if map_cell.id <= MapCellType::UnUse.into_u32() {
            continue;
        }
        //过滤商店
        if map_cell.cell_type == MapCellType::MarketCell {
            continue;
        }
        let mut is_con = false;
        for rem_map_cell in remember_map_cell.iter() {
            //过滤掉记忆队列的地图块
            if map_cell.index != rem_map_cell.cell_index {
                continue;
            }
            is_con = true;
            break;
        }
        if !is_con {
            not_c_v.push(map_cell.index);
        }
        v.push(map_cell.index);
    }
    let mut rand = rand::thread_rng();

    let mut index;
    if !not_c_v.is_empty() {
        index = rand.gen_range(0..not_c_v.len());
        index = *not_c_v.get(index).unwrap();
    } else {
        index = rand.gen_range(0..v.len());
        index = *v.get(index).unwrap();
    }
    index
}

///从记忆队列随机一个地图块
pub fn skill_open_near_cell_robot(
    battle_data: &BattleData,
    robot_data: &RobotData,
) -> Option<usize> {
    let robot_id = robot_data.robot_id;
    let battle_player = battle_data.battle_player.get(&robot_id).unwrap();
    let robot_index = battle_player.get_map_cell_index();

    let (map_cells, _) = battle_data.cal_scope(
        robot_id,
        robot_index as isize,
        TargetType::PlayerSelf,
        None,
        None,
    );

    let mut res_v = vec![];
    for &index in map_cells.iter() {
        let res_index = index as u32;
        let res = battle_data.check_target_array(
            robot_id,
            TargetType::UnOpenMapCellAndUnLock,
            vec![res_index].as_slice(),
        );
        if let Err(_) = res {
            continue;
        }
        res_v.push(index);
    }

    if res_v.is_empty() {
        return None;
    }

    let mut rand = rand::thread_rng();
    //如果记忆队列中小于1个，直接返回
    let remember_map_cell = &robot_data.remember_map_cell;
    if remember_map_cell.is_empty() {
        let index = rand.gen_range(0..res_v.len());
        let index = res_v.get(index).unwrap();
        return Some(*index);
    }
    let mut v = vec![];
    for map_cell in remember_map_cell.iter() {
        for &index in res_v.iter() {
            let cell = battle_data.tile_map.map_cells.get(index).unwrap();
            //排除自己
            if map_cell.cell_index == cell.index {
                continue;
            } else if map_cell.cell_id != cell.id {
                //排除不相等的
                continue;
            }
            v.push(index);
        }
    }
    //如果没找到可以配对的，直接从记忆队列中随机取一个出来
    if v.is_empty() {
        let index = rand.gen_range(0..res_v.len());
        let index = res_v.get(index).unwrap();
        return Some(*index);
    } else {
        let index = rand.gen_range(0..v.len());
        let index = v.get(index).unwrap();
        return Some(*index);
    }
}

///获得除robot_id生命值最高的角色位置
pub fn get_hp_max_cter(battle_data: &BattleData, robot_id: u32) -> Option<usize> {
    let mut res = (0, 0);
    for battle_player in battle_data.battle_player.values() {
        //排除死掉的
        if battle_player.is_died() {
            continue;
        }
        //排除给定robot_id的
        if battle_player.user_id == robot_id {
            continue;
        }
        //对比血量
        if battle_player.cter.base_attr.hp > res.0 {
            res.0 = battle_player.cter.base_attr.hp;
            res.1 = battle_player.cter.index_data.map_cell_index.unwrap();
        }
    }
    //校验返回结果
    if res.1 == 0 {
        return None;
    }
    Some(res.1)
}

///检测是否匹配了
pub fn check_pair(cter: &BattlePlayer) -> bool {
    cter.status.is_pair
}

///有没有相邻的玩家
pub fn no_near_user(battle_data: &BattleData, robot_id: u32) -> bool {
    let battle_player = battle_data.battle_player.get(&robot_id).unwrap();
    let index = battle_player.get_map_cell_index() as isize;
    let res = battle_data.cal_scope(robot_id, index, TargetType::PlayerSelf, None, None);
    res.0.len() > 0
}

///检测是否还有未知地图块，有就随机一块出来并返回
pub fn check_unknow_map_cell(tile_map: &TileMap, robot: &RobotData) -> Option<usize> {
    let mut v = vec![];
    for map_cell in tile_map.map_cells.iter() {
        if map_cell.is_world() {
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
    let rand_index = rand::thread_rng().gen_range(0..v.len());
    let &index = v.get(rand_index).unwrap();
    Some(index)
}

///获得圆形范围aoe范围
pub fn get_roundness_aoe(
    user_id: u32,
    battle_data: &BattleData,
    is_check_null: bool,
    is_check_lock: bool,
    is_opened: bool,
    is_check_world_cell: bool,
) -> Option<Vec<usize>> {
    let mut res_v = vec![];
    for map_cell in battle_data.tile_map.map_cells.iter() {
        //过滤掉无效地图看
        if map_cell.id <= MapCellType::UnUse.into_u32() {
            continue;
        }
        //检查世界块
        if is_check_world_cell && map_cell.is_world() {
            continue;
        }
        //检查是否有锁
        if is_check_lock && map_cell.check_is_locked() {
            continue;
        }
        if is_opened && (map_cell.open_user > 0 || map_cell.pair_index.is_some()) {
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
    for &index in v.iter() {
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
                        if map_cell.is_world() {
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

pub fn can_use_skill(battle_data: &BattleData, battle_player: &BattlePlayer) -> bool {
    let robot = battle_player.robot_data.as_ref().unwrap();
    let battle_data = get_battle_data_ref(battle_player);
    for skill in battle_player.cter.skills.values() {
        let res = skill_condition(battle_data, skill, robot);
        if !res {
            continue;
        }
        let targets = skill_target(battle_data, skill, robot);
        if let Err(_) = targets {
            continue;
        }
        return true;
    }
    false
}
