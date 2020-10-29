use crate::robot::{RememberCell, RobotData};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use tools::macros::GetMutRef;

///触发器类型
#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum RobotTriggerType {
    None = 0,
    SeeMapCell = 1,  //看到地图块
    MapCellPair = 2, //配对地图块
}
impl Default for RobotTriggerType {
    fn default() -> Self {
        RobotTriggerType::None
    }
}

impl RobotTriggerType {
    pub fn into_u8(self) -> u8 {
        let value: u8 = self.into();
        value
    }
}

impl RobotData {
    pub fn trigger_see_map_cell(&self, rc: RememberCell) {
        let self_mut_ref = self.get_mut_ref();
        //如果数量大于5则忘记尾端
        if self_mut_ref.remember_map_cell.len() > 5 {
            self_mut_ref.remember_map_cell.pop_back();
        }
        //如果这个块已经被记忆，则刷新位置
        let mut rm_index = 0_usize;
        for i in self.remember_map_cell.iter() {
            rm_index += 1;
            if i.cell_index == rc.cell_index {
                break;
            }
        }
        self_mut_ref.remember_map_cell.remove(rm_index);
        self_mut_ref.remember_map_cell.push_front(rc);
    }

    pub fn trigger_pair_map_cell(&self, rc: RememberCell) {
        let self_mut_ref = self.get_mut_ref();
        let mut index = 0_usize;
        for i in self.remember_map_cell.iter() {
            if i.cell_index == rc.cell_index {
                break;
            }
            index += 1;
        }
        self_mut_ref.remember_map_cell.remove(index);
    }
}
