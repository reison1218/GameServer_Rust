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
    fn to_insert_vec_value(&mut self) -> Vec<Value>;
    fn to_update_vec_value(&mut self) -> Vec<Value>;
    fn add_version(&mut self);
    fn clear_version(&mut self);
    fn get_version(&self) -> u32;
    fn init(id: u32, js: JsonValue) -> Self;
}

pub trait Dao: Entity {
    fn get_table_name(&mut self) -> &str;
    fn query(user_id: u32, pool: &mut DbPool) -> Option<Self>;

    fn update(&mut self, pool: &mut DbPool) -> Result<u32, String>;

    fn insert(entity: &mut impl Dao, pool: &mut DbPool) -> Result<u32, String>;
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
            return Some(0);
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
            return Some("");
        }
        jv.unwrap().as_str()
    }

    ///获取时间
    fn get_time(&mut self, key: &str) -> Option<NaiveDateTime> {
        let jv = self.get_json_value(key);
        if jv.is_none() {
            return None;
        }
        let str = jv.unwrap().as_str().unwrap();
        let nt = str.parse::<NaiveDateTime>();
        return Some(nt.unwrap());
    }

    ///设置时间
    fn set_time(&mut self, key: String, value: NaiveDateTime) {
        let mut jv = self.get_mut_json_value();
        if jv.is_none() {
            return;
        }
        let value = JsonValue::from(value.format("%Y-%m-%dT%H:%M:%S").to_string());
        jv.unwrap().insert(key, value);
        jv = self.get_mut_json_value();
        println!(
            "{:?}",
            jv.unwrap().get("lastLoginTime").unwrap().as_str().unwrap()
        );
    }

    fn day_reset(&mut self);
}
