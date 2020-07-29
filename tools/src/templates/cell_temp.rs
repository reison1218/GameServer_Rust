use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::{HashMap, HashSet};
use anyhow::Result;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct CellTemp {
    //配置id
    pub id: u32,
    //buff
    pub buff: Vec<u32>,
    ///元素
    pub element: u8,
    ///稀有度
    pub rare:u32,
    ///是否角色块
    pub is_cter:u32,
}

impl Template for CellTemp {}


#[derive(Debug, Default, Clone)]
pub struct CellTempMgr {
    pub temps: HashMap<u32, CellTemp>,//key:id value:celltemp
    pub rare_map:HashMap<u32,HashSet<u32>>,//key:rare value:type list
    pub type_vec:HashMap<u32,HashSet<u32>>,//key:type value:id list
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
        for tt in t {
            let id = tt.id;
            let rare = tt.rare;
            let is_cter = if tt.is_cter == 1{true}else{false};
            let element = tt.element as u32;
            self.temps.insert(tt.id, tt);
            if !self.rare_map.contains_key(&rare){
                self.rare_map.insert(rare,HashSet::new());
            }
            let vec = self.rare_map.get_mut(&rare).unwrap();
            vec.insert(element);

            if !self.type_vec.contains_key(&element){
                self.type_vec.insert(element,HashSet::new());
            }
            let v = self.type_vec.get_mut(&element).unwrap();
            if is_cter{
                continue;
            }
            v.insert(id);
        }
    }
}

impl TemplateMgrTrait for CellTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }
}
