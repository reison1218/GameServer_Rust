use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct SoulTemp {
    pub id: u32,
    pub condition: u32,
}

impl Template for SoulTemp {}

#[derive(Debug, Default, Clone)]
pub struct SoulTempMgr {
    pub temps: HashMap<u32, SoulTemp>,
}

impl SoulTempMgr {
    pub fn init(&mut self, t: Vec<SoulTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }
}

impl TemplateMgrTrait for SoulTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
    }
}
