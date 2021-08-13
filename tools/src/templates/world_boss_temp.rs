use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct WorldBossTemp {
    pub cter_id: u32,
    pub keep_time: u64,
    pub robot_id: u32,
    pub map_ids: Vec<u32>,
}

impl Template for WorldBossTemp {}

#[derive(Debug, Default, Clone)]
pub struct WorldBossTempMgr {
    pub temps: HashMap<u32, WorldBossTemp>,
}

impl WorldBossTempMgr {
    pub fn init(&mut self, t: Vec<WorldBossTemp>) {
        for tt in t {
            self.temps.insert(tt.cter_id, tt);
        }
    }
}

impl TemplateMgrTrait for WorldBossTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
    }
}
