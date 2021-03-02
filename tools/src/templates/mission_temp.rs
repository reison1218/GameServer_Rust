use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

//任务模版
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct MissionTemp {
    pub id: u32,                //任务id
    pub complete_condition: u8, //完成条件
    pub complete_par1: u16,     //完成条件参数1
    pub complete_par2: u16,     //完成条件参数2
    pub complete_par3: u16,     //完成条件参数2
    pub appear_condition: u8,   //出现条件
    pub appear_par1: u16,       //出现参数1
    pub appear_par2: u16,       //出现参数2
    pub appear_par3: u16,       //出现参数3
    pub complete_reward: u16,   //完成奖励
}

impl Template for MissionTemp {}

#[derive(Debug, Default, Clone)]
pub struct MissionTempMgr {
    pub temps: HashMap<u32, MissionTemp>, //key:id value:itemtemp
    pub no_condition_mission: Vec<MissionTemp>, //没有条件的任务
    pub condition_mission: Vec<MissionTemp>, //有条件的任务
}

impl MissionTempMgr {
    #[warn(unreachable_code)]
    pub fn get_temp(&self, id: &u32) -> anyhow::Result<&MissionTemp> {
        let res = self.temps.get(id);
        if res.is_none() {
            let str = format!("ItemTemp is none for id:{}", id);
            anyhow::bail!(str)
        };
        Ok(res.unwrap())
    }

    pub fn init(&mut self, t: Vec<MissionTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt.clone());
            if tt.complete_condition == 0 {
                self.no_condition_mission.push(tt.clone());
            } else {
                self.condition_mission.push(tt)
            }
        }
    }

    pub fn condition_mission(&self) -> &[MissionTemp] {
        &self.condition_mission
    }

    pub fn no_condition_mission(&self) -> &[MissionTemp] {
        &self.no_condition_mission
    }
}

impl TemplateMgrTrait for MissionTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
        self.no_condition_mission.clear();
        self.condition_mission.clear();
    }
}
