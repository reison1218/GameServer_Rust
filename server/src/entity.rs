pub mod contants;
pub mod user;
use crate::db::dbtool::DbPool;
use chrono::NaiveDateTime;
use mysql::prelude::ToValue;
use mysql::Value;
use serde_json::{Map, Value as JsonValue};
use std::cell::Cell;
use std::ops::Add;
use std::str::FromStr;

pub trait Entity: Clone {
    fn to_vec_value(&mut self) -> Vec<Value>;
    fn add_version(&mut self);
    fn clear_version(&mut self);
    fn get_version(&self) -> u32;
    fn init(id: u32, js: JsonValue) -> Self;
}

pub trait Dao: Entity {
    fn query(user_id: u32, pool: &mut DbPool) -> Option<Self>;

    fn update(&mut self, pool: &mut DbPool) -> Result<u32, String>;
}

pub trait Data {
    fn get_json_value(&mut self, key: &str) -> Option<&JsonValue>;

    fn get_mut_json_value(&mut self) -> Option<&mut Map<String, JsonValue>>;

    ///获取玩家id
    fn get_id(&self) -> Option<u32>;

    ///设置玩家id
    fn set_id(&mut self, id: u32);

    ///获得u64数据，弱不存在这个key，则返回None
    fn get_usize(&mut self, key: &str) -> Option<usize> {
        let jv = self.get_json_value(key);
        if jv.is_none() {
            return None;
        }
        Some(jv.unwrap().as_u64().unwrap() as usize)
    }

    fn set_usize(&mut self, key: String, value: usize) {
        let map: Option<&mut Map<String, JsonValue>> = self.get_mut_json_value();
        if map.is_none() {
            return;
        }
        let v = JsonValue::from(value);
        map.unwrap().insert(key, v);
    }

    ///获得字符串切片引用，若不存在这个key，则返回""
    fn get_str(&mut self, key: &str) -> Option<&str> {
        let jv = self.get_json_value(key);
        if jv.is_none() {
            return None;
        }
        jv.unwrap().as_str()
    }

    ///获取时间
    fn get_time(&mut self, key: &str) -> Option<NaiveDateTime> {
        let jv = self.get_json_value(key);
        if jv.is_none() {
            return None;
        }
        let nt = chrono::NaiveDateTime::from_str(jv.unwrap().as_str().unwrap());
        return Some(nt.unwrap());
    }

    ///设置时间
    fn set_time(&mut self, key: String, value: NaiveDateTime) {
        let jv = self.get_mut_json_value();
        if jv.is_none() {
            return;
        }
        let value = JsonValue::from(value.to_string());
        jv.unwrap().insert(key, value);
    }
}
