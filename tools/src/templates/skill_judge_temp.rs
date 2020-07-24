use std::collections::HashMap;
use crate::templates::template::{Template, TemplateMgrTrait};

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct SkillJudgeTemp {
    pub id: u32,//技能id
    pub target:u32,//目标
    pub par1: u32,//参数1
    pub par2:u32,//参数2
    pub par3:u32,//参数3
    pub par4:u32,//参数4
    pub par5:u32,//参数5
}

impl Template for SkillJudgeTemp {}



#[derive(Debug, Default, Clone)]
pub struct SkillJudgeTempMgr {
    pub temps: HashMap<u32, SkillJudgeTemp>,//key:id value:celltemp
}

impl SkillJudgeTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&SkillJudgeTemp> {
        let res = self.temps.get(id);
        if res.is_none(){
            let str = format!("SkillJudgeTemp is none for id:{}",id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<SkillJudgeTemp>) {
        for tt in t{
            self.temps.insert(tt.id,tt);
        }
    }
}

impl TemplateMgrTrait for SkillJudgeTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }
}