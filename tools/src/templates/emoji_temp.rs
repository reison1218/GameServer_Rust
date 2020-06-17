use crate::templates::template::Template;
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize, Debug, Default, Clone)]
pub struct EmojiTemp {
    pub id: u32,
    pub condition: u32,
}

impl Template for EmojiTemp{}


#[derive(Debug, Default, Clone)]
pub struct EmojiTempMgr{
    pub temps: HashMap<u32, EmojiTemp>,
}

impl EmojiTempMgr{
    pub fn init(&mut self, t: Vec<EmojiTemp>) {
        for tt in t {
            self.temps.insert(tt.id, tt);
        }
    }
}