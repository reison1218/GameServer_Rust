use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct ItemTemp {
    pub id: u32,            //技能id
    pub trigger_skill: u32, //触发的技能
}

impl Template for ItemTemp {}

#[derive(Debug, Default, Clone)]
pub struct ItemTempMgr {
    pub temps: HashMap<u32, ItemTemp>, //key:id value:itemtemp
}

impl ItemTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&ItemTemp> {
        let res = self.temps.get(id);
        if res.is_none() {
            let str = format!("ItemTemp is none for id:{}", id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<ItemTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }
}

impl TemplateMgrTrait for ItemTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
    }
}
