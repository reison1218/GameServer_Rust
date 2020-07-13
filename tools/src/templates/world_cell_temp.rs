use std::collections::HashMap;
use crate::templates::template::Template;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct WorldCellTemp{
    pub id: u32,
    pub skill_id: Vec<u32>,
}

impl Template for WorldCellTemp{}


#[derive(Debug, Default, Clone)]
pub struct WorldCellTempMgr {
    pub temps: HashMap<u32, WorldCellTemp>,
}

impl  WorldCellTempMgr{

    pub fn init(&mut self, t: Vec<WorldCellTemp>) {
        for tt in t {
           self.temps.insert(tt.id,tt);
        }
    }

}
