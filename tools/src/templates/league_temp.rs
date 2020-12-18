use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

///段位配置
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct LeagueTemp {
    id: u8,
    score: u32,
}

impl Template for LeagueTemp {}

#[derive(Debug, Default, Clone)]
pub struct LeagueTempMgr {
    pub temps: HashMap<u8, LeagueTemp>, //key:id value:itemtemp
}

impl TemplateMgrTrait for LeagueTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
    }
}

impl LeagueTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u8) -> anyhow::Result<&LeagueTemp> {
        let res = self.temps.get(id);
        if res.is_none() {
            let str = format!("LeagueTemp is none for id:{}", id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<LeagueTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }
}
