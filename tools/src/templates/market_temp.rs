use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct MarketTemp {
    pub id: u32,           //技能id
    pub market_count: u32, //商品数量
}

impl Template for MarketTemp {}

#[derive(Debug, Default, Clone)]
pub struct MarketTempMgr {
    pub temps: HashMap<u32, MarketTemp>, //key:id value:itemtemp
}

impl MarketTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&MarketTemp> {
        let res = self.temps.get(id);
        if res.is_none() {
            let str = format!("ItemTemp is none for id:{}", id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<MarketTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }
}

impl TemplateMgrTrait for MarketTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
    }
}
