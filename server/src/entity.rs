pub mod character;
pub mod character_contants;
pub mod user;
pub mod user_contants;
pub mod user_info;
use crate::entity::user_info::User;
use crate::mgr::game_mgr::GameMgr;
use crate::DB_POOL;
use chrono::NaiveDateTime;
use log::{error, info, warn};
use mysql::prelude::ToValue;
use mysql::{Error, QueryResult, Value};
use serde_json::{Map, Value as JsonValue};
use std::any::Any;
use std::sync::{Arc, Mutex};

///关于结构体转换的trait
pub trait Entity: Send + Sync {
    ///将自身转换成mysql到value，用于进行mysql的数据库操作
    fn to_insert_vec_value(&self) -> Vec<Value> {
        let mut v: Vec<Value> = Vec::new();
        v.push(self.get_user_id().to_value());
        let tem_id = self.get_tem_id();
        if tem_id.is_some() {
            v.push(Value::from(tem_id.unwrap()));
        }
        v.push(Value::from(self.get_data().to_string()));
        v
    }
    ///将自身转换成mysql到value，用于进行mysql的数据库操作
    fn to_update_vec_value(&self) -> Vec<Value> {
        let mut v: Vec<Value> = Vec::new();
        v.push(Value::from(self.get_data().to_string()));
        v.push(self.get_user_id().to_value());
        let tem_id = self.get_tem_id();
        if tem_id.is_some() {
            v.push(Value::from(tem_id.unwrap()));
        }
        v
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
    ///获得u64数据，弱不存在这个key，则返回None
    fn get_usize(&self, key: &str) -> Option<usize> {
        let jv = self.get_json_value(key);
        if jv.is_none() {
            return Some(0);
        }
        Some(jv.unwrap().as_u64().unwrap() as usize)
    }

    ///设置usize数据类型
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

    ///根据key获得json数据，返回一个jsonvalue结构体只读指针
    fn get_json_value(&self, key: &str) -> Option<&JsonValue> {
        let v = self.get_data().as_object().unwrap().get(key);
        if v.is_none() {
            return None;
        }
        v
    }

    ///根据key获得json数据，返回一个jsonvalue结构体可变指针
    fn get_mut_json_value(&mut self) -> Option<&mut Map<String, JsonValue>> {
        self.get_data_mut().as_object_mut()
    }
    ///设置玩家id
    fn set_user_id(&mut self, user_id: u32);

    ///设置主键id
    fn set_ids(&mut self, user_id: u32, tem_id: u32);

    ///设置时间
    fn set_time(&mut self, key: String, value: NaiveDateTime) {
        let jv = self.get_mut_json_value();
        if jv.is_none() {
            return;
        }
        let value = JsonValue::from(value.format("%Y-%m-%dT%H:%M:%S").to_string());
        jv.unwrap().insert(key, value);
    }

    ///更新登陆时间
    fn update_login_time(&mut self);

    ///更新下线时间
    fn update_off_time(&mut self);

    ///每日重制（由time_mgr中的定时器调用）
    fn day_reset(&mut self);
    ///添加版本号
    fn add_version(&mut self);
    ///清空版本号
    fn clear_version(&mut self);
    ///获得版本号
    fn get_version(&self) -> u32;
    ///获得配置id（静态表的）
    fn get_tem_id(&self) -> Option<u32>;
    ///获取玩家id
    fn get_user_id(&self) -> u32;
    ///获得数据
    fn get_data(&self) -> &JsonValue;
    ///获得数据
    fn get_data_mut(&mut self) -> &mut JsonValue;
    ///初始化函数，注意，这里函数返回地方加上了where从句限定方式，用于规避"trait object safe"问题
    /// 当使用"trait object"的时候，只允许"?Sized"的类型数据，并且函数前面参数部分必须包含self参数
    /// 这里加上从句是让编译器在处理"trait object"的时候，无视这个函数。
    fn init(user_id: u32, tem_id: Option<u32>, js: JsonValue) -> Self
    where
        Self: Sized;
}

///关于结构体DB操作的trait
pub trait Dao: Entity {
    ///获得表名
    fn get_table_name(&mut self) -> &str;

    ///更新函数（trait默认函数，不必重写）
    fn update(&mut self) -> Result<u32, String> {
        let v: Vec<Value> = self.to_update_vec_value();
        let mut sql = String::new();
        sql.push_str("update ");
        sql.push_str(self.get_table_name());
        sql.push_str(" set content=:data where user_id=:user_id ");
        let tem_id = self.get_tem_id();
        if tem_id.is_some() {
            sql.push_str("and tem_id=:tem_id");
        }
        let qr: Result<QueryResult, Error> = DB_POOL.exe_sql(sql.as_str(), Some(v));
        if qr.is_err() {
            let err = qr.err().unwrap();
            error!("{:?}", err);
            return Err(err.to_string());
        }
        self.clear_version();
        Ok(1)
    }

    ///insert函数（trait默认函数，不必重写）
    fn insert(&mut self) -> Result<u32, String> {
        let v: Vec<Value> = self.to_insert_vec_value();
        let mut sql = String::new();
        sql.push_str("insert into ");
        sql.push_str(self.get_table_name());
        let tem_id = self.get_tem_id();

        match tem_id {
            Some(_) => {
                sql.push_str(" values(:user_id,:tem_id,:content)");
            }
            None => {
                sql.push_str(" values(:user_id,:content)");
            }
        }

        let qr: Result<QueryResult, Error> = DB_POOL.exe_sql(sql.as_str(), Some(v));

        if qr.is_err() {
            let str = String::from_utf8(qr.unwrap().info());
            let s = str.unwrap();
            println!("{:?}", s);
            return Err(s);
        }
        Ok(1)
    }
}

///作为trait object
pub trait EntityData: Dao + Any {
    ///深拷贝函数
    fn try_clone(&self) -> Box<dyn EntityData>;
}

///提供给http保存玩家数据的函数
pub fn save_player_http(gm: Arc<Mutex<GameMgr>>) {
    let gm = gm.clone();
    let mut gm = gm.lock().unwrap();
    gm.save_user_http();
}
