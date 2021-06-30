use crate::{battle::battle::BattleData, room::map_data::MapCell};

pub fn check_can_open(map_cell: &MapCell, battle_data: &BattleData) -> bool {
    if map_cell.check_is_locked() {
        return false;
    }
    let user_id = map_cell.user_id;
    if user_id == 0 {
        return true;
    }

    let player = battle_data.battle_player.get(&user_id).unwrap();
    player.can_be_move()
}

pub fn modify_robot_state(robot_id: u32, bm: &mut BattleData) {
    let robot = bm.battle_player.get_mut(&robot_id).unwrap();
    let robot_data = robot.robot_data.as_mut().unwrap();
    robot_data.is_action = true;
}
