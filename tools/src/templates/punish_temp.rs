use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct PunishTemp {
    pub id: u32,          //id
    pub punish_time: i64, //惩罚时间
}

impl Template for PunishTemp {}

#[derive(Debug, Default, Clone)]
pub struct PunishTempMgr {
    pub max_id: u32,
    pub temps: HashMap<u32, PunishTemp>, //key:id value:PunishTemp
}

impl PunishTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&PunishTemp> {
        let res;
        if *id > self.max_id {
            res = self.temps.get(&self.max_id);
        } else {
            res = self.temps.get(id);
        }
        if res.is_none() {
            let str = format!("PunishTemp is none for id:{}", id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<PunishTemp>) {
        for tt in t {
            if tt.id > self.max_id {
                self.max_id = tt.id;
            }
            self.temps.insert(tt.id, tt);
        }
    }
}

impl TemplateMgrTrait for PunishTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
    }
}
