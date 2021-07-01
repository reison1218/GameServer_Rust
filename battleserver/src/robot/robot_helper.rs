use crate::{battle::battle::BattleData, room::map_data::MapCell};

pub fn check_can_open(user_id: u32, map_cell: &MapCell, battle_data: &BattleData) -> bool {
    if map_cell.check_is_locked() {
        return false;
    }
    let user = map_cell.user_id;
    if user == user_id || user == 0 {
        return true;
    }

    let player = battle_data.battle_player.get(&user).unwrap();
    player.can_be_move()
}
