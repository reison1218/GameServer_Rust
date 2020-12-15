use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct BuffTemp {
    pub id: u32,           //技能id
    pub target: u8,        //目标类型
    pub keep_time: u8,     //持续时间
    pub trigger_times: u8, //触发次数
    pub scope: u32,        //范围
    pub par1: u32,         //参数1
    pub par2: u32,         //参数2
    pub par3: u32,         //参数3
    pub par4: u32,         //参数3
    pub par5: u32,         //参数3
}

impl Template for BuffTemp {}

#[derive(Debug, Default, Clone)]
pub struct BuffTempMgr {
    pub temps: HashMap<u32, BuffTemp>, //key:id value:BuffTemp
}

impl BuffTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&BuffTemp> {
        let res = self.temps.get(id);
        if res.is_none() {
            let str = format!("BuffTemp is none for id:{}", id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<BuffTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }
}

impl TemplateMgrTrait for BuffTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
    }
}
