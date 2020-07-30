use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;
use anyhow::Result;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct TileMapTemp {
    pub id: u32,
    pub map: Vec<u32>,
    pub cell_rare: Vec<CellRare>,
    pub world_cell: u32,
    pub map_type: u8,
    pub member_count:u32,
}

impl Template for TileMapTemp {}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct CellRare {
    pub rare: u32,
    pub count: u32,
}

#[derive(Debug, Default, Clone)]
pub struct TileMapTempMgr {
    pub temps: HashMap<u32, TileMapTemp>,
    ///key:member_count key:is_has_world_cell
    pub member_temps:HashMap<u32,HashMap<bool,Vec<TileMapTemp>>>,
}

impl TileMapTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, map_id: u32) -> Result<&TileMapTemp> {
        let res = self.temps.get(&map_id);
        if res.is_none(){
            let str = format!("TileMapTemp is none for map_id:{}",map_id);
           anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<TileMapTemp>) {
        for tt in t {
            if !self.member_temps.contains_key(&tt.member_count){
                self.member_temps.insert(tt.member_count,HashMap::new());
            }
            let map = self.member_temps.get_mut(&tt.member_count).unwrap();
            let res = tt.world_cell>0;
            if !map.contains_key(&res){
                map.insert(res,Vec::new());
            }
            map.get_mut(&res).unwrap().push(tt.clone());
            self.temps.insert(tt.id, tt);

        }
    }
}

impl TemplateMgrTrait for TileMapTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }
}
