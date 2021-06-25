use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

///机器人配置
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct RobotTemp {
    id: u32,
    pub cter_id: u32,
    pub skills: Vec<u32>,
}

impl Template for RobotTemp {}

impl RobotTemp {
    pub fn get_id(&self) -> u32 {
        self.id
    }
}

///角色配置管理器
#[derive(Debug, Default, Clone)]
pub struct RobotTempMgr {
    pub temps: HashMap<u32, RobotTemp>,
    pub cter_ids: Vec<u32>,
    pub cters: HashMap<u32, Vec<RobotTemp>>,
}

impl RobotTempMgr {
    pub fn init(&mut self, t: Vec<RobotTemp>) {
        let v: Vec<RobotTemp> = t;
        for ct in v {
            if !self.cter_ids.contains(&ct.id) {
                self.cter_ids.push(ct.id);
            }
            if !self.cters.contains_key(&ct.id) {
                self.cters.insert(ct.id, vec![]);
            }
            let v = self.cters.get_mut(&ct.id).unwrap();
            v.push(ct.clone());
            self.temps.insert(ct.id, ct);
        }
    }
    pub fn get_temp_ref(&self, id: &u32) -> Option<&RobotTemp> {
        self.temps.get(id)
    }
}

impl TemplateMgrTrait for RobotTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
        self.cters.clear();
    }
}
