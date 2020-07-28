use std::collections::HashMap;
use crate::templates::template::{Template, TemplateMgrTrait};

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct SkillScopeTemp {
    pub id: u32,//技能id
    pub scope:Vec<DirectionTemp>,//范围
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct DirectionTemp{
    pub direction:Vec<i32>,
}

impl Template for SkillScopeTemp {}



#[derive(Debug, Default, Clone)]
pub struct SkillScopeTempMgr {
    pub temps: HashMap<u32, SkillScopeTemp>,//key:id value:celltemp
}

impl SkillScopeTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&SkillScopeTemp> {
        let res = self.temps.get(id);
        if res.is_none(){
            let str = format!("SkillScopeTemp is none for id:{}",id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<SkillScopeTemp>) {
        for tt in t{
            self.temps.insert(tt.id,tt);
        }
    }
}

impl TemplateMgrTrait for SkillScopeTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }
}