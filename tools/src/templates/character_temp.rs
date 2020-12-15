use crate::templates::template::{Template, TemplateMgrTrait};
use std::borrow::Borrow;
use std::collections::HashMap;

///角色配置结构体
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct CharacterTemp {
    pub id: u32,
    pub hp: i16,
    pub attack: u8,
    pub defence: u8,
    pub start_energy: u8,
    pub max_energy: u8,
    pub element: u8,
    pub skills: Vec<Group>,
    pub passive_buff: Vec<u32>,
    pub lock_skills: Vec<Group>,
    pub is_dlc: u8,
    pub is_init: u8,
    pub usable_skill_count: u8,
    pub usable_item_count: u8,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct Group {
    pub group: Vec<u32>,
}

impl Template for CharacterTemp {}

impl CharacterTemp {
    pub fn get_id(&self) -> u32 {
        self.id
    }
}

///角色配置管理器
#[derive(Debug, Default, Clone)]
pub struct CharacterTempMgr {
    pub name: String,
    pub temps: HashMap<u32, CharacterTemp>,
    pub init_temps: Vec<CharacterTemp>,
}

impl CharacterTempMgr {
    pub fn get_init_character(&self) -> &Vec<CharacterTemp> {
        self.init_temps.borrow()
    }
    pub fn init(&mut self, t: Vec<CharacterTemp>) {
        let v: Vec<CharacterTemp> = t;
        for ct in v {
            if ct.is_init == 1 {
                self.init_temps.push(ct.clone());
            }
            self.temps.insert(ct.id, ct);
        }
    }
    pub fn get_temp_ref(&self, id: &u32) -> Option<&CharacterTemp> {
        self.temps.get(id)
    }
}

impl TemplateMgrTrait for CharacterTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }

    fn clear(&mut self) {
        self.temps.clear();
        self.init_temps.clear();
    }
}
