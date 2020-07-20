use std::collections::HashMap;
use crate::templates::template::{Template, TemplateMgrTrait};

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct TriggerTimeTemp {
    pub id: u32,//技能id
    pub par1: u32,//参数1
    pub par2:u32,//参数2
    pub par3:u32,//参数3
    pub par4:u32,//参数4
    pub par5:u32,//参数5
}

impl Template for TriggerTimeTemp {}



#[derive(Debug, Default, Clone)]
pub struct TriggerTimeTempMgr {
    pub temps: HashMap<u32, TriggerTimeTemp>,//key:id value:celltemp
}

impl TriggerTimeTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&TriggerTimeTemp> {
        let res = self.temps.get(id);
        if res.is_none(){
            let str = format!("TriggerTimeTemp is none for id:{}",id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<TriggerTimeTemp>) {
        for tt in t{
            self.temps.insert(tt.id,tt);
        }
    }
}

impl TemplateMgrTrait for TriggerTimeTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }
}