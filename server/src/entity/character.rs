use super::*;
use crate::entity::character_contants::{GRADE, LAST_USE_SKILLS, SKILLS};
use crate::TEMPLATES;
use std::collections::HashMap;
use std::str::FromStr;
use tools::protos::base::CharacterPt;
use tools::templates::character_temp::CharacterTempMgr;
use tools::templates::template::TemplateMgrTrait;

#[derive(Debug, Clone, Default)]
pub struct Characters {
    pub user_id: u32,                      //玩家id
    pub cter_map: HashMap<u32, Character>, //玩家角色
    pub version: u32,                      //版本号
}

impl Characters {
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
            version: 0,
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

    pub fn get_need_update_array(&mut self) -> Vec<Box<dyn EntityData>> {
        let mut v: Vec<Box<dyn EntityData>> = Vec::new();
        for (_, cter) in self.cter_map.iter_mut() {
            if cter.version > 0 {
                cter.version = 0;
                v.push(cter.try_clone());
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
            let (uid, tid, js) = mysql::from_row(_qr.unwrap());
            let c = Character::init(uid, Some(tid), js);
            map.insert(c.character_id, c);
        }
        if map.is_empty() {
            return None;
        }
        let mut c = Characters::default();
        c.user_id = user_id;
        c.cter_map = map;
        c.version = 0;
        Some(c)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Character {
    pub user_id: u32,      //玩家id
    pub character_id: u32, //角色id
    pub data: JsonValue,   //数据
    pub version: u32,      //数据版本号
}

impl Into<CharacterPt> for Character {
    fn into(self) -> CharacterPt {
        let mut cter_pt = CharacterPt::default();
        let res = self.get_skills();
        match res {
            Ok(skills) => {
                cter_pt.set_skills(skills);
            }
            Err(_) => {}
        }

        cter_pt.set_cter_id(self.character_id);
        let res = self.get_grade();
        match res {
            Ok(grade) => {
                cter_pt.set_grade(grade);
            }
            Err(_) => {
                cter_pt.set_grade(1);
            }
        }
        let last_use_skills = self.get_last_use_skills().unwrap();
        cter_pt.set_last_use_skills(last_use_skills);
        cter_pt
    }
}

impl Character {
    pub fn new(user_id: u32, character_id: u32, js: JsonValue) -> Self {
        let mut cter = Character::init(user_id, Some(character_id), js);
        let res = TEMPLATES
            .get_constant_ref()
            .temps
            .get("character_init_grade");
        let grade;
        match res {
            Some(temp) => {
                let s = usize::from_str(temp.value.as_str());
                match s {
                    Ok(g) => {
                        grade = g;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        grade = 1_usize;
                    }
                }
            }
            None => {
                grade = 1_usize;
            }
        }
        cter.set_usize(GRADE.to_string(), grade);
        cter
    }

    pub fn get_skills(&self) -> anyhow::Result<Vec<u32>> {
        let mut v: Vec<u32> = Vec::new();
        let res = self.get_json_value(SKILLS);
        if res.is_none() {
            return Ok(v);
        }
        let json = res.unwrap();
        v = serde_json::from_value(json.clone())?;
        Ok(v)
    }
    pub fn set_skills(&mut self, skills: Vec<u32>) {
        let map: Option<&mut Map<String, JsonValue>> = self.get_mut_json_value();
        if map.is_none() {
            return;
        }
        let v = JsonValue::from(skills);
        map.unwrap().insert(SKILLS.to_owned(), v);
    }

    pub fn get_last_use_skills(&self) -> anyhow::Result<Vec<u32>> {
        let mut v: Vec<u32> = Vec::new();
        let res = self.get_json_value(LAST_USE_SKILLS);
        if res.is_none() {
            return Ok(v);
        }
        let json = res.unwrap();
        v = serde_json::from_value(json.clone())?;
        Ok(v)
    }

    pub fn get_grade(&self) -> anyhow::Result<u32> {
        let res = self.get_json_value(GRADE);
        if res.is_none() {
            return Ok(0);
        }
        let res = res.unwrap().as_u64();
        let res = res.unwrap() as u32;
        Ok(res)
    }

    #[warn(dead_code)]
    pub fn query(table_name: &str, user_id: u32, tem_id: Option<u32>) -> Option<Self> {
        let mut v: Vec<Value> = Vec::new();
        v.push(Value::UInt(user_id as u64));

        let mut sql = String::new();
        sql.push_str("select * from ");
        sql.push_str(table_name);
        sql.push_str(" where user_id=:user_id");
        if tem_id.is_some() {
            sql.push_str(" and tem_id:tem_id");
        }

        let q: Result<QueryResult, Error> = DB_POOL.exe_sql(sql.as_str(), Some(v));
        if q.is_err() {
            error!("{:?}", q.err().unwrap());
            return None;
        }
        let q = q.unwrap();

        let mut data = None;
        for _qr in q {
            let (id, js) = mysql::from_row(_qr.unwrap());
            let c = Character::init(id, tem_id, js);
            data = Some(c);
        }
        data
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

    fn day_reset(&mut self) {
        unimplemented!()
    }

    fn add_version(&mut self) {
        self.version += 1;
    }

    fn clear_version(&mut self) {
        self.version = 0;
    }

    fn get_version(&self) -> u32 {
        self.version
    }

    fn get_tem_id(&self) -> Option<u32> {
        Some(self.character_id)
    }

    fn get_user_id(&self) -> u32 {
        self.user_id
    }

    fn get_data(&self) -> &JsonValue {
        &self.data
    }

    fn get_data_mut(&mut self) -> &mut JsonValue {
        &mut self.data
    }

    fn init(user_id: u32, tem_id: Option<u32>, js: JsonValue) -> Self
    where
        Self: Sized,
    {
        let c = Character {
            user_id,
            character_id: tem_id.unwrap(),
            data: js,
            version: 0 as u32,
        };
        c
    }
}

impl EntityData for Character {
    fn try_clone(&self) -> Box<dyn EntityData> {
        let cter = Character::init(
            self.get_user_id(),
            Some(self.character_id),
            self.data.clone(),
        );
        Box::new(cter)
    }
}

impl Dao for Character {
    fn get_table_name(&mut self) -> &str {
        "t_u_character"
    }
}

fn get_init_characters(user_id: u32) -> Result<Vec<Character>, String> {
    let mut v: Vec<Character> = Vec::new();
    let cter_temp: &CharacterTempMgr = TEMPLATES.get_character_ref();
    if cter_temp.is_empty() {
        error!("there are no Character templates!");
        return Err("there are no Character templates!".to_string());
    }
    let characters = cter_temp.get_init_character();
    for c in characters {
        let mut map = Map::new();
        let skill_array = JsonValue::from(c.skills.clone());
        map.insert(
            SKILLS.to_owned(),
            serde_json::Value::from(skill_array.clone()),
        );

        let jv = serde_json::Value::from(map);
        let cter = Character::new(user_id, c.get_id() as u32, jv);
        v.push(cter);
    }
    Ok(v)
}
