use crate::{battle::battle::BattleData, room::map_data::MapCell};

pub fn check_can_open(cter_id: u32, map_cell: &MapCell, battle_data: &BattleData) -> bool {
    if map_cell.check_is_locked() {
        return false;
    }
    let map_cter_id = map_cell.cter_id;
    if map_cter_id == cter_id || map_cter_id == 0 {
        return true;
    }

    let cter = battle_data.get_battle_cter(map_cter_id, true).unwrap();
    cter.can_be_move()
}
