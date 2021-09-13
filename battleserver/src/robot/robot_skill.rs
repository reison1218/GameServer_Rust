use std::collections::HashMap;

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

use super::robot_task_mgr::RobotTask;
use super::{RobotActionType, RobotData};

///机器人使用技能
pub fn robot_use_skill(battle_data: &mut BattleData, skill: &Skill, robot: &RobotData) -> bool {
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
        warn!("{:?}", targets.err().unwrap());
        return false;
    }
    let targets = targets.unwrap();
    let skill_id = skill.id;
    //创建机器人任务执行
    let mut robot_task = RobotTask::default();
    robot_task.action_type = RobotActionType::Skill;
    robot_task.robot_id = robot_id;
    let mut map = Map::new();
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
    if skill.function_id == 431 {
        return false;
    }
    let skill_id = skill.id;

    let skill_function_id = skill.function_id;

    let mut can_use = false;
    let robot_id = robot.robot_id;
    let robot_player = battle_data.get_battle_player(Some(robot_id), true).unwrap();
    let cter_id = robot_player.current_cter.0;
    let skill_judge = skill.skill_temp.skill_judge as u32;
    //如果cd好了就设置状态
    if skill.cd_times == 0 {
        can_use = true;
    }

    let battle_player = battle_data.battle_player.get(&robot_id).unwrap();
    let cter = battle_player.get_current_cter();

    let res = check_skill_useable(cter, skill);
    if let Err(_) = res {
        return false;
    }

    let res = battle_data.check_skill_judge(cter_id, skill_judge, Some(skill_id), None);
    if let Err(_) = res {
        return false;
    }
    //特殊使用条件
    match skill_function_id {
        //判断是否有未知地图快
        i if 113 == i => {
            // can_use = check_unknow_map_cell(&battle_data.tile_map, robot).is_some();
        }
        //判断是否配对
        i if 211 == i => {
            let battle_player = battle_data.battle_player.get(&robot_id).unwrap();
            can_use = battle_player.is_can_attack();
        }
        //判断有没有地图块可以翻
        i if 223 == i => {
            let targets = skill_target(battle_data, skill, robot);
            if let Err(_) = targets {
                can_use = false;
            } else {
                can_use = true;
            }
        }
        //周围必须没人
        i if 313 == i => {
            can_use = !near_user(battle_data, cter_id);
        }
        //周围必须有人
        i if 321 == i => {
            can_use = near_user(battle_data, cter_id);
        }
        i if 331 == i => {
            let battle_player = battle_data.battle_player.get(&robot_id).unwrap();
            can_use = pair_useable_skill(battle_player);
        }
        //选中至少2个目标
        i if 411 == i => {
            let res = get_line_aoe(robot_id, battle_data);
            match res {
                Some((count, _)) => {
                    let alive_num = battle_data.get_alive_player_num();

                    if count > 1 {
                        can_use = true;
                    } else if count == 1 && alive_num == 2 {
                        can_use = true;
                    } else {
                        can_use = false;
                    }
                }
                None => {
                    can_use = false;
                }
            }
        }
        12002 => {
            //如果激活中就直接返回
            if skill.is_active {
                return false;
            }
            //有其他技能可以用就直接返回
            for cter_skill in cter.skills.values() {
                if cter_skill.cd_times == 0 {
                    return false;
                }
            }
            //如果还可以攻击就返回
            if battle_player.is_can_attack() {
                return false;
            }
            //还可以移动就返回
            if battle_player.flow_data.residue_movement_points > 0 {
                return false;
            }
        }
        13001 => {
            if !skill.is_active {
                return false;
            }
            for &minon_id in cter.minons.iter() {
                let minon = battle_data.get_battle_cter(minon_id, true);
                if minon.is_err() {
                    continue;
                }
                let minon = minon.unwrap();
                let minon_index = minon.get_map_cell_index();
                let res = battle_data.cal_scope(
                    minon_id,
                    minon_index as isize,
                    TargetType::MapCellEnemys,
                    None,
                    None,
                );
                if !res.1.is_empty() {
                    return true;
                }
            }
        }
        13004 => {
            can_use = !cter.minons.is_empty();
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
    let skill_function_id = skill.function_id;
    let robot_id = robot.robot_id;

    let battle_robot = battle_data.get_battle_player(Some(robot_id), true).unwrap();

    let cter_id = battle_robot.current_cter.0;
    let team_id = battle_robot.team_id;
    let mut targets = vec![];
    //匹配技能id进行不同的目标选择
    match skill_function_id {
        //目标是自己
        i if [211, 313, 321].contains(&i) => {
            // targets.push(battle_player.get_current_cter_mut().index_data.map_cell_index.unwrap());
        }
        //除自己外最大血量的目标
        i if [123, 331, 433, 20001, 20002, 20003, 20004, 20005].contains(&i) => {
            let res = get_hp_max_cter(battle_data, robot_id, None);
            match res {
                Some(res) => targets.push(res),
                None => warn!("get_hp_max_cter res is None!"),
            }
        }
        //随机未知地图块
        i if [113].contains(&i) => {
            // let res = check_unknow_map_cell(&battle_data.tile_map, robot);
            // if let Some(index) = res {
            //     targets.push(index);
            // }
        }
        //获得记忆队列中的地图块
        i if [223].contains(&i) => {
            let res = skill_open_near_cell_robot(battle_data, robot);
            if let Some(res) = res {
                targets.push(res);
            }
        }
        //直线三个aoe
        i if [411].contains(&i) => {
            let res = get_line_aoe(cter_id, battle_data);
            match res {
                Some((_, v)) => {
                    targets.extend_from_slice(v.as_slice());
                }
                None => {
                    warn!("get_triangle_aoe could not find any target!")
                }
            }
        }
        //随机不在记忆队列中的地图块
        i if [423].contains(&i) => {
            let res = rand_not_remember_map_cell(&battle_data.tile_map, robot);
            if let Some(index) = res {
                targets.push(index);
            }
        }
        //变身技能，计算⭕️
        i if [431].contains(&i) => {
            let res =
                get_roundness_aoe(cter_id, battle_data, true, false, true, true, Some(team_id));
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
            let res = get_roundness_aoe(
                cter_id,
                battle_data,
                false,
                false,
                false,
                false,
                Some(team_id),
            );
            match res {
                Some(res) => {
                    targets.extend_from_slice(res.as_slice());
                }
                None => {
                    warn!("get_roundness_aoe could not find any target!")
                }
            }
        }
        11004 => {
            let mut v = vec![];
            for &cter_id in battle_data.cter_player.keys() {
                let cter = battle_data.get_battle_cter(cter_id, true);
                if cter.is_err() {
                    continue;
                }
                let cter = cter.unwrap();
                if !cter.is_major {
                    continue;
                }
                if cter.get_cter_id() == skill.last_target_cter
                    && battle_data.battle_player.len() > 2
                {
                    continue;
                }
                if cter.base_attr.team_id == team_id {
                    continue;
                }
                v.push(cter.get_map_cell_index());
            }
            let mut random = rand::thread_rng();
            let index = random.gen_range(0..v.len());
            let &target_cter_index = v.get(index).unwrap();
            targets.push(target_cter_index);
        }
        11002 => {}
        11005 => {
            let target_cter_index = get_hp_max_cter(battle_data, robot_id, Some(team_id));
            if let Some(target_cter_index) = target_cter_index {
                targets.push(target_cter_index);
            }
        }
        11007 => {
            let res = get_triangle_aoe(cter_id, battle_data, Some(team_id));
            if let Some(res) = res {
                targets.extend_from_slice(res.as_slice());
            }
        }
        11008 => {
            let res = get_roundness_aoe(
                cter_id,
                battle_data,
                false,
                false,
                false,
                false,
                Some(team_id),
            );
            if let Some(res) = res {
                targets.extend_from_slice(res.as_slice());
            }
        }
        11009 => {
            for &id in battle_data.cter_player.keys() {
                let cter_res = battle_data.get_battle_cter(id, true);
                match cter_res {
                    Ok(cter) => {
                        if cter.base_attr.team_id == team_id {
                            continue;
                        } else {
                            targets.push(cter.get_map_cell_index());
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        12001 => {
            //⭕️aoe，包括中心，人数越多越好
            let res = get_roundness_aoe(
                cter_id,
                battle_data,
                false,
                false,
                false,
                false,
                Some(team_id),
            );
            match res {
                Some(res) => {
                    targets.extend_from_slice(res.as_slice());
                }
                None => {
                    warn!("get_roundness_aoe could not find any target!")
                }
            }
        }
        12002 => {
            let last_cter_id = skill.last_target_cter;
            let mut cter_id;
            let mut res_v = vec![];
            for player in battle_data.battle_player.values() {
                if player.team_id == team_id {
                    continue;
                }
                cter_id = player.major_cter.0;
                if cter_id == last_cter_id {
                    continue;
                }

                let cter = battle_data.get_battle_cter(cter_id, true);
                match cter {
                    Ok(cter) => {
                        res_v.push(cter.get_map_cell_index());
                    }
                    Err(_) => continue,
                }
            }
            if res_v.is_empty() {
                for player in battle_data.battle_player.values() {
                    if player.team_id == team_id {
                        continue;
                    }
                    cter_id = player.major_cter.0;
                    let cter = battle_data.get_battle_cter(cter_id, true);
                    match cter {
                        Ok(cter) => {
                            res_v.push(cter.get_map_cell_index());
                        }
                        Err(_) => continue,
                    }
                }
            }
            let mut random = rand::thread_rng();
            let random_index = random.gen_range(0..res_v.len());
            let &index = res_v.get(random_index).unwrap();
            targets.push(index);
        }
        12003 => {
            let cter = battle_data.get_battle_cter(cter_id, true).unwrap();
            targets.push(cter.get_map_cell_index());
        }
        12004 => {
            let target_cter_index = get_hp_min_cter(battle_data, Some(team_id));
            if let Some(target_cter_index) = target_cter_index {
                targets.push(target_cter_index);
            }
        }
        13002 => {
            let res = get_nearest_cter(battle_data, cter_id, Some(team_id));
            if let Some(index) = res {
                targets.push(index);
            }
        }
        13004 => {
            let target_cter_index = get_hp_max_cter(battle_data, robot_id, Some(team_id));
            if let Some(target_cter_index) = target_cter_index {
                targets.push(target_cter_index);
            }
        }
        13005 => {
            let res = battle_data.get_enemys(team_id);
            for id in res {
                let cter_res = battle_data.get_battle_cter(id, true);
                if cter_res.is_err() {
                    continue;
                }
                let cter_res = cter_res.unwrap();
                targets.push(cter_res.get_map_cell_index());
            }
        }
        _ => {}
    }
    Ok(targets)
}

///随机一个不在记忆队列中的地图块
pub fn rand_not_remember_map_cell(tile_map: &TileMap, robot: &RobotData) -> Option<usize> {
    let remember_map_cell = &robot.remember_map_cell;

    let mut not_c_v = vec![];
    let mut v = vec![];
    for (&map_cell_index, _) in tile_map.un_pair_map.iter() {
        for rem_map_cell in remember_map_cell.iter() {
            //过滤掉记忆队列的地图块
            if map_cell_index == rem_map_cell.cell_index {
                continue;
            }
            not_c_v.push(map_cell_index);
        }
        v.push(map_cell_index);
    }
    let mut rand = rand::thread_rng();

    let index;
    let rand_index;
    if !not_c_v.is_empty() {
        rand_index = rand.gen_range(0..not_c_v.len());
        index = Some(*not_c_v.get(rand_index).unwrap());
    } else {
        rand_index = rand.gen_range(0..v.len());
        index = Some(*v.get(rand_index).unwrap());
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
    let robot_index = battle_player.get_current_cter_index();

    let (map_cells, _) = battle_data.cal_scope(
        battle_player.current_cter.0,
        robot_index as isize,
        TargetType::PlayerSelf,
        None,
        None,
    );

    let mut res_v = vec![];
    for &index in map_cells.iter() {
        let map_cell = battle_data.tile_map.map_cells.get(index).unwrap();
        if map_cell.open_cter > 0 {
            continue;
        }
        if map_cell.check_is_locked() {
            continue;
        }

        if map_cell.cell_type != MapCellType::Valid {
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
        Some(*index)
    } else {
        let index = rand.gen_range(0..v.len());
        let index = v.get(index).unwrap();
        Some(*index)
    }
}

pub fn get_nearest_cter(
    battle_data: &BattleData,
    cter_id: u32,
    team_id: Option<u8>,
) -> Option<usize> {
    let cter = battle_data.get_battle_cter(cter_id, true);
    if cter.is_err() {
        return None;
    }
    let cter = cter.unwrap();
    let cter_index = cter.get_map_cell_index();
    let map_cell = battle_data.tile_map.map_cells.get(cter_index).unwrap();

    let mut res_v = vec![];

    let res = map_cell.x + map_cell.y;
    let mut team = 0;
    if let Some(team_id) = team_id {
        team = team_id;
    }
    for &id in battle_data.cter_player.keys() {
        let other_cter = battle_data.get_battle_cter(id, true);
        if other_cter.is_err() {
            continue;
        }
        let other_cter = other_cter.unwrap();
        if other_cter.base_attr.team_id == team {
            continue;
        }

        let other_cell = battle_data
            .tile_map
            .map_cells
            .get(other_cter.get_map_cell_index());
        if other_cell.is_none() {
            continue;
        }
        let other_cell = other_cell.unwrap();
        let value = res - (other_cell.x + other_cell.y);
        let value_res = value.abs();
        let res = (value_res, other_cter.get_map_cell_index());
        res_v.push(res);
    }
    let min_value = res_v.iter().min_by(|a, b| a.cmp(b));
    if min_value.is_none() {
        return None;
    }
    let res = min_value.unwrap();
    Some(res.1)
}

///获得生命值最低点角色
pub fn get_hp_min_cter(battle_data: &BattleData, team_id: Option<u8>) -> Option<usize> {
    let mut res = (0, 0);
    for &cter_id in battle_data.cter_player.values() {
        let cter = battle_data.get_battle_cter(cter_id, true);
        if cter.is_err() {
            continue;
        }
        let cter = cter.unwrap();
        if team_id.is_some() && (team_id.unwrap() == cter.base_attr.team_id) {
            continue;
        }
        if res.0 == 0 {
            res.0 = cter.base_attr.hp;
            res.1 = cter.get_map_cell_index();
        }
        if res.0 < cter.base_attr.hp {
            res.0 = cter.base_attr.hp;
            res.1 = cter.get_map_cell_index();
        }
    }
    //校验返回结果
    if res.1 == 0 {
        return None;
    }
    Some(res.1)
}

///获得除robot_id生命值最高的角色位置
pub fn get_hp_max_cter(
    battle_data: &BattleData,
    robot_id: u32,
    team_id: Option<u8>,
) -> Option<usize> {
    let mut res = (0, 0);
    for battle_player in battle_data.battle_player.values() {
        //排除死掉的
        if battle_player.is_died() {
            continue;
        }
        //排除自己所有的角色
        if battle_player.get_user_id() == robot_id {
            continue;
        }
        if let Some(team_id) = team_id {
            if battle_player.team_id == team_id {
                continue;
            }
        }
        for battle_cter in battle_player.cters.values() {
            let hp = battle_cter.base_attr.hp;
            //对比血量
            if hp > res.0 {
                res.0 = hp;
                res.1 = battle_cter.get_map_cell_index();
            }
        }
    }
    //校验返回结果
    if res.1 == 0 {
        return None;
    }
    Some(res.1)
}

pub fn pair_useable_skill(robot: &BattlePlayer) -> bool {
    robot.flow_data.pair_usable_skills.contains(&331)
}

///有没有相邻的玩家
pub fn near_user(battle_data: &BattleData, cter_id: u32) -> bool {
    let battle_cter = battle_data.get_battle_cter(cter_id, true).unwrap();
    let index = battle_cter.get_map_cell_index() as isize;
    let res = battle_data.cal_scope(cter_id, index, TargetType::PlayerSelf, None, None);
    !res.1.is_empty()
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
    if v.is_empty() {
        return None;
    }
    let rand_index = rand::thread_rng().gen_range(0..v.len());
    let &index = v.get(rand_index).unwrap();
    Some(index)
}

///获得圆形范围aoe范围
pub fn get_roundness_aoe(
    cter_id: u32,
    battle_data: &BattleData,
    is_check_null: bool,
    is_check_lock: bool,
    is_opened: bool,
    is_check_world_cell: bool,
    team_id: Option<u8>,
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
        if is_opened && (map_cell.open_cter > 0 || map_cell.pair_index.is_some()) {
            continue;
        }
        let mut v = vec![];
        if is_check_null && map_cell.cter_id > 0 {
            continue;
        } else if map_cell.cter_id == cter_id {
            //排除自己
            continue;
        } else if map_cell.cter_id > 0 {
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
            let cter_id = cell.cter_id;
            if cter_id == 0 {
                continue;
            }

            let cter = battle_data.get_battle_cter(cter_id, true);
            if cter.is_err() {
                continue;
            }
            let cter = cter.unwrap();

            //一个队伍就跳过
            if team_id.is_some() && (cter.base_attr.team_id == team_id.unwrap()) {
                continue;
            }
            v.push(cell.index);
        }
        res_v.push(v);
    }
    res_v.iter().max().cloned()
}

///获得三角aoe范围
pub fn get_triangle_aoe(
    cter_id: u32,
    battle_data: &BattleData,
    team_id: Option<u8>,
) -> Option<Vec<usize>> {
    let mut res_v = vec![];
    for map_cell in battle_data.tile_map.map_cells.iter() {
        let mut v = vec![];
        //过滤掉无效地图块
        if map_cell.id <= MapCellType::UnUse.into_u32() {
            continue;
        }
        if map_cell.cter_id == cter_id {
            continue;
        }
        //把中心点加进去
        if map_cell.cter_id > 0 {
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
            if res_cell.cter_id == 0 {
                continue;
            }
            let cter = battle_data.get_battle_cter(res_cell.cter_id, true);
            if cter.is_err() {
                continue;
            }
            let cter = cter.unwrap();

            //排除队友
            if team_id.is_some() && (team_id.unwrap() == cter.base_attr.team_id) {
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
pub fn get_line_aoe(cter_id: u32, battle_data: &BattleData) -> Option<(u32, Vec<usize>)> {
    let mut v = Vec::new();
    let map_cells = &battle_data.tile_map.map_cells;
    for index in 0..map_cells.len() {
        let cell = map_cells.get(index).unwrap();
        if cell.cter_id == 0 {
            continue;
        }
        if cell.cter_id == cter_id {
            continue;
        }
        v.push(index);
    }

    let mut res_v = HashMap::new();
    for &index in v.iter() {
        let map = battle_data.tile_map.map_cells.get(index).unwrap();
        //从六个方向计算
        for i in 0..6 {
            let mut temp_v = (1, vec![]);
            temp_v.1.push(index);
            //先把起点的人加进去
            //每个方向从中心点延伸出去两个格子
            for j in 0..2 {
                let (mut coord_index_x, mut coord_index_y) = (map.x, map.y);
                match i {
                    0 => match j {
                        0 => {
                            coord_index_x -= 1;
                            coord_index_y += 1;
                        }
                        1 => {
                            coord_index_x -= 2;
                            coord_index_y += 2;
                        }
                        _ => {}
                    },
                    1 => match j {
                        0 => {
                            coord_index_y += 1;
                        }
                        1 => {
                            coord_index_y += 2;
                        }
                        _ => {}
                    },
                    2 => match j {
                        0 => {
                            coord_index_x += 1;
                        }
                        1 => {
                            coord_index_x += 2;
                        }
                        _ => {}
                    },
                    3 => match j {
                        0 => {
                            coord_index_x += 1;
                            coord_index_x -= 1;
                        }
                        1 => {
                            coord_index_x += 2;
                            coord_index_x -= 2;
                        }
                        _ => {}
                    },
                    4 => match j {
                        0 => {
                            coord_index_x -= 1;
                        }
                        1 => {
                            coord_index_x -= 2;
                        }
                        _ => {}
                    },
                    5 => match j {
                        0 => {
                            coord_index_x += 1;
                        }
                        1 => {
                            coord_index_x += 2;
                        }
                        _ => {}
                    },
                    _ => {}
                }
                let coord_index = (coord_index_x, coord_index_y);
                let res = battle_data.tile_map.coord_map.get(&coord_index);
                if res.is_none() {
                    continue;
                }
                let index = *res.unwrap();
                let map_cell = battle_data.tile_map.map_cells.get(index);
                match map_cell {
                    Some(map_cell) => {
                        //不能是世界树
                        if map_cell.is_world() {
                            continue;
                        }

                        if map_cell.cell_type != MapCellType::Valid {
                            continue;
                        }

                        if map_cell.cter_id == cter_id {
                            continue;
                        }

                        if temp_v.1.contains(&map_cell.index) {
                            continue;
                        }
                        if map_cell.cter_id > 0 {
                            temp_v.0 += 1;
                        }
                        temp_v.1.push(map_cell.index);
                    }
                    None => {}
                }
            }
            if !res_v.contains_key(&temp_v.0) {
                res_v.insert(temp_v.0, Vec::new());
            }
            res_v.get_mut(&temp_v.0).unwrap().push(temp_v.1);
        }
    }
    if res_v.is_empty() {
        return None;
    }
    let mut res_1 = vec![];
    let mut res_2 = vec![];
    let mut res_3 = vec![];
    for (&count, v) in res_v.iter() {
        if count == 1 {
            res_1.push(v.clone());
        } else if count == 2 {
            res_2.push(v.clone());
        } else if count == 3 {
            res_3.push(v.clone());
        }
    }
    if !res_3.is_empty() {
        let mut rand = rand::thread_rng();
        let mut index = rand.gen_range(0..res_3.len());
        let mut res = res_3.remove(index);

        index = rand.gen_range(0..res.len());
        let res = res.remove(index);
        let mut fin_res = vec![];
        fin_res.push(*res.get(1).unwrap());
        fin_res.push(*res.get(0).unwrap());
        let last = res.get(2);
        if let Some(&last) = last {
            fin_res.push(last);
        }

        return Some((3, fin_res));
    } else if !res_2.is_empty() {
        let mut rand = rand::thread_rng();
        let mut index = rand.gen_range(0..res_2.len());
        let mut res = res_2.remove(index);

        index = rand.gen_range(0..res.len());
        let res = res.remove(index);
        let mut fin_res = vec![];

        fin_res.push(*res.get(1).unwrap());
        fin_res.push(*res.get(0).unwrap());
        let last = res.get(2);
        if let Some(&last) = last {
            fin_res.push(last);
        }
        return Some((2, fin_res));
    } else if !res_1.is_empty() {
        let mut rand = rand::thread_rng();
        let mut index = rand.gen_range(0..res_1.len());
        let mut res = res_1.remove(index);

        index = rand.gen_range(0..res.len());
        let res = res.remove(index);
        let mut fin_res = vec![];

        let mut last = res.get(1);
        if let Some(&last) = last {
            fin_res.push(last);
        }
        fin_res.push(*res.get(0).unwrap());
        last = res.get(2);
        if let Some(&last) = last {
            fin_res.push(last);
        }
        Some((1, fin_res))
    } else {
        None
    }
}

pub fn can_use_skill(battle_data: &BattleData, battle_player: &BattlePlayer) -> bool {
    let robot = battle_player.robot_data.as_ref().unwrap();
    for skill in battle_player.get_current_cter().skills.values() {
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
