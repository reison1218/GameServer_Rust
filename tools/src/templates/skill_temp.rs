use std::collections::HashMap;
use crate::templates::template::{Template, TemplateMgrTrait};

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct SkillTemp {
    pub id: u32,//技能id
    pub skill_judge: u16,//判定条件
    pub consume_type:u8,//消耗类型
    pub consume_value:u16,//消耗值
    pub cd:u8,//cd
    pub keep_time:i8,//持续轮次数
    pub scope:u32,//范围
    pub trigger_time:u16,//触发条件
}

impl Template for SkillTemp {}



#[derive(Debug, Default, Clone)]
pub struct SkillTempMgr {
    pub temps: HashMap<u32, SkillTemp>,//key:id value:celltemp
    pub lock_skills:Vec<u32>,
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
            if tt.id == 321{
                self.lock_skills.push(tt.id);
            }
            self.temps.insert(tt.id,tt);
        }
    }
}

impl TemplateMgrTrait for SkillTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }
}