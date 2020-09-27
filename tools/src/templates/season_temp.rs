use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct SeasonTemp {
    pub id: u32,     //赛季id
    pub element: u8, //赛季元素
}

impl Template for SeasonTemp {}

#[derive(Debug, Default, Clone)]
pub struct SeasonTempMgr {
    pub temps: HashMap<u32, SeasonTemp>, //key:id value:SeasonTemp
}

impl SeasonTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&SeasonTemp> {
        let res = self.temps.get(id);
        if res.is_none() {
            let str = format!("SeasonTemp is none for id:{}", id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<SeasonTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }
}

impl TemplateMgrTrait for SeasonTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }
}
