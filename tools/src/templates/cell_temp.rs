use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::{HashMap, HashSet};
use anyhow::Result;
use std::cell::Cell;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct CellTemp {
    pub id: u32,
    pub buff_id: Vec<u32>,
    pub cell_type: u32,
    pub rare:u32,
    pub is_cter:u32
}

impl Template for CellTemp {}


#[derive(Debug, Default, Clone)]
pub struct CellTempMgr {
    pub temps: HashMap<u32, CellTemp>,//key:id value:celltemp
    pub rare_map:HashMap<u32,HashSet<u32>>,//key:rare value:type list
    pub type_vec:HashMap<u32,HashSet<CellTemp>>,//key:type value:celltemp list
}

impl CellTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> Result<&CellTemp> {
        let res = self.temps.get(id);
        if res.is_none(){
            let str = format!("CellTemp is none for id:{}",id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<CellTemp>) {
        // for tt in t {
        //     let id = tt.id;
        //     self.temps.insert(tt.id, tt.clone());
        //     if !self.rare_map.contains_key(&tt.rare){
        //         self.rare_map.insert(tt.rare,HashSet::new());
        //     }
        //     let vec = self.rare_map.get_mut(&tt.rare).unwrap();
        //     vec.insert(tt.cell_type);
        //
        //     if !self.type_vec.contains_key(&tt.cell_type){
        //         self.type_vec.insert(tt.cell_type,HashSet::new());
        //     }
        //     let v = self.type_vec.get_mut(&tt.cell_type).unwrap();
        //     if tt.is_cter == 1{
        //         continue;
        //     }
        //     v.insert(tt);
        // }
    }
}

impl TemplateMgrTrait for CellTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }
}
