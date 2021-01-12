use crate::templates::template::{Template, TemplateMgrTrait};
use anyhow::Result;
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct TileMapTemp {
    pub id: u32,
    pub map: Vec<u32>,
    pub cell_rare: Vec<CellRare>,
    pub world_cell: u32,
    pub member_count: Vec<u32>,
    pub season_id: u32,
    pub world_cell_index: u32,
    pub member_count_key: u8,
}

impl Template for TileMapTemp {}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct CellRare {
    pub rare: u16,
    pub count: u32,
}

#[derive(Debug, Default, Clone)]
pub struct TileMapTempMgr {
    pub temps: HashMap<u32, TileMapTemp>,
    ///key:member_count key:is_has_world_cell
    pub member_temps: HashMap<u8, HashMap<bool, Vec<TileMapTemp>>>,
    ///key:赛季id key:人数 keu:位置 value:vec<TileMapTemp>
    pub season_temps: HashMap<u32, HashMap<u8, HashMap<u32, Vec<TileMapTemp>>>>,
}

impl TileMapTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, map_id: u32) -> Result<&TileMapTemp> {
        let res = self.temps.get(&map_id);
        if res.is_none() {
            let str = format!("TileMapTemp is none for map_id:{}", map_id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<TileMapTemp>) {
        for tt in t {
            if !self.member_temps.contains_key(&tt.member_count_key) {
                self.member_temps
                    .insert(tt.member_count_key, HashMap::new());
            }
            let map = self.member_temps.get_mut(&tt.member_count_key).unwrap();
            let res = tt.world_cell > 0;
            if !map.contains_key(&res) {
                map.insert(res, Vec::new());
            }
            map.get_mut(&res).unwrap().push(tt.clone());

            if !self.season_temps.contains_key(&tt.season_id) {
                self.season_temps.insert(tt.season_id, HashMap::new());
            }
            let map = self.season_temps.get_mut(&tt.season_id).unwrap();
            if !map.contains_key(&tt.member_count_key) {
                map.insert(tt.member_count_key, HashMap::new());
            }

            let index_map = map.get_mut(&tt.member_count_key).unwrap();
            if !index_map.contains_key(&(tt.world_cell_index as u32)) {
                index_map.insert(tt.world_cell_index, Vec::new());
            }
            index_map
                .get_mut(&tt.world_cell_index)
                .unwrap()
                .push(tt.clone());

            self.temps.insert(tt.id, tt);
        }
    }
}

impl TemplateMgrTrait for TileMapTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
        self.member_temps.clear();
        self.season_temps.clear();
    }
}
