use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

///结算奖励配置
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct SummaryAwardTemp {
    pub id: u8,
    pub score: i16,
}
impl Template for SummaryAwardTemp {}

#[derive(Debug, Default, Clone)]
pub struct SummaryAwardTempMgr {
    pub temps: HashMap<u8, SummaryAwardTemp>, //key:id value:itemtemp
}

impl TemplateMgrTrait for SummaryAwardTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
    }
}

impl SummaryAwardTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u8) -> anyhow::Result<&SummaryAwardTemp> {
        let res = self.temps.get(id);
        if res.is_none() {
            let str = format!("SummaryAwardTemp is none for id:{}", id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<SummaryAwardTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }

    pub fn get_score_by_rank(&self, rank: u8) -> anyhow::Result<i16> {
        let res = self.get_temp(&rank)?;
        Ok(res.score)
    }
}
