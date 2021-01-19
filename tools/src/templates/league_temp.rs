use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

///段位配置
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct LeagueTemp {
    pub id: i8,
    pub score: i32,
}

impl Template for LeagueTemp {}

#[derive(Debug, Default, Clone)]
pub struct LeagueTempMgr {
    pub temps: HashMap<i8, LeagueTemp>, //key:id value:itemtemp
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
    pub fn get_temp(&self, id: &i8) -> anyhow::Result<&LeagueTemp> {
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

    pub fn get_league_by_score(&self, score: i32) -> anyhow::Result<&LeagueTemp> {
        let mut res_temp = None;
        for temp in self.temps.values() {
            if score < temp.score {
                continue;
            }
            res_temp = Some(temp);
        }
        if let None = res_temp {
            anyhow::bail!("can not find temp for score:{}!", score)
        }
        Ok(res_temp.unwrap())
    }
}
