use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

///段位配置
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct BattleLimitTimeTemp {
    pub id: u8,
    pub ms: u32,
}

impl Template for BattleLimitTimeTemp {}

#[derive(Debug, Default, Clone)]
pub struct BattleLimitTimeTempMgr {
    pub temps: HashMap<u8, BattleLimitTimeTemp>, //key:id value:itemtemp
}

impl TemplateMgrTrait for BattleLimitTimeTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
    }
}

impl BattleLimitTimeTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u8) -> anyhow::Result<&BattleLimitTimeTemp> {
        let res = self.temps.get(id);
        if res.is_none() {
            let str = format!("BattleLimitTimeTemp is none for id:{}", id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<BattleLimitTimeTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }
}
