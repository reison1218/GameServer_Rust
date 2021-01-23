use super::*;
use crate::TEMPLATES;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::HashMap;
use tools::protos::base::CharacterPt;
use tools::templates::character_temp::{CharacterTempMgr, Group};
use tools::templates::template::TemplateMgrTrait;

#[derive(Debug, Clone, Default)]
pub struct Characters {
    pub user_id: u32,                      //玩家id
    pub cter_map: HashMap<u32, Character>, //玩家角色
    pub version: Cell<u32>,                //版本号
}

impl Characters {
    pub fn add_use_times(&mut self, cter_id: u32) -> Vec<u32> {
        let res = self.cter_map.get_mut(&cter_id);
        if res.is_none() {
            return Vec::new();
        }
        let cter = res.unwrap();
        cter.use_times += 1;
        cter.add_version();
        let mut v = Vec::new();
        self.cter_map.iter().for_each(|(cter_id, cter)| {
            v.push((*cter_id, cter.use_times));
        });
        v.par_sort_unstable_by(|a, b| b.1.cmp(&a.1));
        self.add_version();
        let mut res_v = Vec::new();
        for (cter_id, use_count) in v {
            if use_count<=0{
                continue;
            }
            if res_v.len()>=3{
                break;
            }
            res_v.push(cter_id);
        }
        res_v
    }
    fn add_version(&self) {
        let res = self.version.get() + 1;
        self.version.set(res);
    }

    pub fn get_frist(&self) -> u32 {
        let mut cter_id = 1001_u32;
        for i in self.cter_map.iter() {
            cter_id = *i.0;
            break;
        }
        cter_id
    }

    pub fn new(user_id: u32) -> Self {
        let cter_map: HashMap<u32, Character> = HashMap::new();
        let mut cters = Characters {
            user_id,
            cter_map,
            version: Cell::new(0),
        };
        let res = get_init_characters(user_id);
        if res.is_ok() {
            let res = res.unwrap();
            for c in res {
                cters.cter_map.insert(c.character_id, c);
            }
        }
        cters
    }

    pub fn get_need_update_array(&self) -> Vec<Box<dyn EntityData>> {
        let mut v: Vec<Box<dyn EntityData>> = Vec::new();
        for (_, cter) in self.cter_map.iter() {
            if cter.version.get() > 0 {
                v.push(cter.try_clone_for_db());
            }
        }
        v
    }

    pub fn query(table_name: &str, user_id: u32) -> Option<Self> {
        let mut v: Vec<Value> = Vec::new();
        v.push(Value::UInt(user_id as u64));

        let mut sql = String::new();
        sql.push_str("select * from ");
        sql.push_str(table_name);
        sql.push_str(" where user_id=:user_id");

        let q: Result<QueryResult, Error> = DB_POOL.exe_sql(sql.as_str(), Some(v));
        if q.is_err() {
            error!("{:?}", q.err().unwrap());
            return None;
        }
        let mut map = HashMap::new();
        let q = q.unwrap();
        for _qr in q {
            let (_, _, data): (u32, u32, serde_json::Value) = mysql::from_row(_qr.unwrap());
            let c = Character::init(data);
            map.insert(c.character_id, c);
        }
        if map.is_empty() {
            return None;
        }
        let mut c = Characters::default();
        c.user_id = user_id;
        c.cter_map = map;
        c.version = Cell::new(0);
        Some(c)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Character {
    pub user_id: u32,              //玩家id
    pub character_id: u32,         //角色id
    pub use_times: u32,            //角色使用次数
    pub skills: Vec<Group>,        //技能
    pub last_use_skills: Vec<u32>, //上次使用的技能
    #[serde(skip_serializing)]
    pub version: Cell<u32>, //数据版本号
}

impl Into<CharacterPt> for Character {
    fn into(self) -> CharacterPt {
        let mut cter_pt = CharacterPt::default();
        let res = self.get_skills();
        cter_pt.set_skills(res);
        cter_pt.set_cter_id(self.character_id);
        let last_use_skills = self.get_last_use_skills();
        cter_pt.set_last_use_skills(last_use_skills);
        cter_pt
    }
}

impl Character {
    pub fn new(user_id: u32, character_id: u32, skills: Vec<Group>) -> Self {
        let mut cter = Character::default();
        cter.user_id = user_id;
        cter.character_id = character_id;
        cter.skills = skills;
        cter
    }

    pub fn get_skills(&self) -> Vec<u32> {
        let mut v = Vec::new();
        for group in self.skills.iter() {
            v.extend_from_slice(&group.group[..])
        }
        v
    }

    pub fn get_last_use_skills(&self) -> Vec<u32> {
        self.last_use_skills.clone()
    }
}

impl Entity for Character {
    fn set_user_id(&mut self, user_id: u32) {
        self.user_id = user_id;
    }

    fn set_ids(&mut self, user_id: u32, tem_id: u32) {
        self.user_id = user_id;
        self.character_id = tem_id;
    }

    fn update_login_time(&mut self) {
        unimplemented!()
    }

    fn update_off_time(&mut self) {
        unimplemented!()
    }

    fn day_reset(&mut self) {
        unimplemented!()
    }

    fn add_version(&self) {
        let res = self.version.get() + 1;
        self.version.set(res);
    }

    fn clear_version(&self) {
        self.version.set(0);
    }

    fn get_version(&self) -> u32 {
        self.version.get()
    }

    fn get_tem_id(&self) -> Option<u32> {
        Some(self.character_id)
    }

    fn get_user_id(&self) -> u32 {
        self.user_id
    }

    fn get_data(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    fn init(data: serde_json::Value) -> Self
    where
        Self: Sized,
    {
        let c = serde_json::from_value(data).unwrap();
        c
    }
}

impl EntityData for Character {
    fn try_clone_for_db(&self) -> Box<dyn EntityData> {
        let res = Box::new(self.clone());
        self.version.set(0);
        res
    }
}

impl Dao for Character {
    fn get_table_name(&self) -> &str {
        "t_u_character"
    }
}

fn get_init_characters(user_id: u32) -> Result<Vec<Character>, String> {
    let mut v: Vec<Character> = Vec::new();
    let cter_temp: &CharacterTempMgr = TEMPLATES.get_character_temp_mgr_ref();
    if cter_temp.is_empty() {
        error!("there are no Character templates!");
        return Err("there are no Character templates!".to_string());
    }
    let characters = cter_temp.get_init_character();
    for c in characters {
        let mut skill_v = Vec::new();
        for group in c.skills.iter() {
            skill_v.push(group.clone());
        }
        let cter = Character::new(user_id, c.get_id() as u32, skill_v);
        v.push(cter);
    }
    Ok(v)
}
