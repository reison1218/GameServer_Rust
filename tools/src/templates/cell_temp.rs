use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;
use anyhow::Result;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct CellTemp {
    pub id: u32,
    pub buff_id: Vec<u32>,
    pub cell_type: u32,
    pub rare:u32,
}

impl Template for CellTemp {}


#[derive(Debug, Default, Clone)]
pub struct CellTempMgr {
    pub temps: HashMap<u32, CellTemp>,
}

impl CellTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, map_id: u32) -> Result<&CellTemp> {
        let res = self.temps.get(&map_id);
        if res.is_none(){
            let str = format!("CellTemp is none for map_id:{}",map_id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<CellTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }
}

impl TemplateMgrTrait for CellTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }
}
