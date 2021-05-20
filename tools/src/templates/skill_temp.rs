use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct SkillTemp {
    pub id: u32,           //技能id
    pub function_id: u32,  //关联id
    pub skill_judge: u16,  //判定条件
    pub target: u8,        //目标类型
    pub par1: u32,         //参数1
    pub par2: u32,         //参数2
    pub par3: u32,         //参数3
    pub par4: u32,         //参数4
    pub consume_type: u8,  //消耗类型
    pub consume_value: u8, //消耗值
    pub cd: u8,            //cd
    pub scope: u32,        //范围
    pub buff: u32,         //能够触发的buff
    pub view_target: u8,   //视野目标
}

impl Template for SkillTemp {}

#[derive(Debug, Default, Clone)]
pub struct SkillTempMgr {
    pub temps: HashMap<u32, SkillTemp>, //key:id value:celltemp
    pub lock_skills: Vec<u32>,
}

impl SkillTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&SkillTemp> {
        let res = self.temps.get(id);
        if res.is_none() {
            let str = format!("SkillTemp is none for id:{}", id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<SkillTemp>) {
        let mut id;
        for tt in t {
            id = tt.id;
            if tt.id == 321 {
                self.lock_skills.push(id);
            }
            self.temps.insert(id, tt);
        }
    }
}

impl TemplateMgrTrait for SkillTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
        self.lock_skills.clear();
    }
}
