use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct WorldCellTemp {
    pub id: u32,
    pub buff: Vec<u32>,
}

impl Template for WorldCellTemp {}

#[derive(Debug, Default, Clone)]
pub struct WorldCellTempMgr {
    pub temps: HashMap<u32, WorldCellTemp>,
}

impl WorldCellTempMgr {
    pub fn init(&mut self, t: Vec<WorldCellTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }
}

impl TemplateMgrTrait for WorldCellTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
    }
}
