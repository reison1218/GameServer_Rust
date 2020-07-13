use std::collections::HashMap;
use crate::templates::template::{Template, TemplateMgrTrait};

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct SkillTemp {
    pub id: u32,//技能id
    pub target_type: u8,//作用目标类型
    pub effect_type:u8,//效果类型
    pub effect_value:u16,//效果值
    pub consume_type:u8,//消耗类型
    pub consume_value:u16,//消耗值
    pub cd:u8,//cd
    pub keep_time:u8,//持续轮次数
    pub scope:u32,//范围
    pub trigger_condition:u8,//触发条件
    pub trgger_value:Vec<u32>,//触发的技能
}

impl Template for SkillTemp {}



#[derive(Debug, Default, Clone)]
pub struct SkillTempMgr {
    pub temps: HashMap<u32, SkillTemp>,//key:id value:celltemp
}

impl SkillTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&SkillTemp> {
        let res = self.temps.get(id);
        if res.is_none(){
            let str = format!("SkillTemp is none for id:{}",id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<SkillTemp>) {
        for tt in t{
            self.temps.insert(tt.id,tt);
        }
    }
}

impl TemplateMgrTrait for SkillTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }
}