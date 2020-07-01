use crate::templates::template::{Template, TemplateMgrTrait};
use std::borrow::Borrow;
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct CterCell{
    cell_id:u32,
    count:u32,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct CharacterTemp {
    pub id: u32,
    pub hp_max: u16,
    pub attack: u16,
    pub skill_consume: u8,
    pub skill_value: u16,
    pub skills: Vec<u32>,
    pub lock_skills: Vec<u32>,
    pub is_dlc: u8,
    pub is_init: u8,
    pub cter_cell:CterCell
}

impl Template for CharacterTemp {}

impl CharacterTemp {
    pub fn get_id(&self) -> u32 {
        self.id
    }
}

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
    pub fn get_temp_ref(&self,id:&u32)->Option<&CharacterTemp>{
        self.temps.get(id)
    }
}

impl TemplateMgrTrait for CharacterTempMgr {
    fn is_empty(&self) -> bool {
        self.temps.is_empty()
    }
}
