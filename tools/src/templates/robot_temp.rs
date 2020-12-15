use crate::templates::character_temp::Group;
use crate::templates::template::{Template, TemplateMgrTrait};
use std::collections::HashMap;

///机器人配置
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct RobotTemp {
    id: u32,
    pub skills: Vec<Group>,
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
    pub cters: Vec<u32>,
}

impl RobotTempMgr {
    pub fn init(&mut self, t: Vec<RobotTemp>) {
        let v: Vec<RobotTemp> = t;
        for ct in v {
            self.cters.push(ct.id);
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
