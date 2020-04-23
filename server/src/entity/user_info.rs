use super::*;
use crate::entity::contants::*;
use chrono::Local;
use std::collections::HashMap;

///玩家基本数据结构体，用于封装例如玩家ID，昵称，创建时间等等
/// user_id:玩家ID
/// data：作为玩家具体数据，由jsonvalue封装
/// version：数据版本号，大于0则代表有改动，需要update到db
#[derive(Debug, Clone,Default)]
pub struct User {
    pub user_id: u32,    //玩家id
    pub data: JsonValue, //数据
    pub version: u32,    //数据版本号
}

///为User实现Entiry
impl Entity for User{
    fn to_insert_vec_value(&self) -> Vec<Value> {
        let mut v: Vec<Value> = Vec::new();
        v.push(self.user_id.to_value());
        v.push(Value::from(self.data.to_string()));
        v
    }

    fn to_update_vec_value(&self) -> Vec<Value> {
        let mut v: Vec<Value> = Vec::new();
        v.push(Value::from(self.data.to_string()));
        v.push(self.user_id.to_value());
        v
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
        None
    }

    fn init(user_id: u32, tem_id: Option<u32>, js: JsonValue) -> Self {
        let u = User {
            user_id: user_id,
            data: js,
            version: 0,
        };
        u
    }
}

impl EntityData for User {
    fn try_clone(&self) -> Box<EntityData> {
        let user = User::init(self.get_user_id(),None,self.data.clone());
        Box::new(user)
    }

    fn get_json_value(&mut self, key: &str) -> Option<&JsonValue> {
        let v = self.data.as_object_mut().unwrap().get(key);
        if v.is_none() {
            return None;
        }
        v
    }

    fn get_mut_json_value(&mut self) -> Option<&mut Map<String, JsonValue>> {
        self.data.as_object_mut()
    }

    ///获取玩家id
    fn get_user_id(&self) -> u32 {
        self.user_id
    }

    ///设置玩家id
    fn set_user_id(&mut self, user_id: u32) {
        self.user_id = user_id;
        self.add_version();
    }

    ///设置玩家id
    fn set_ids(&mut self, user_id: u32,tem_id:u32) {
        self.user_id = user_id;
        self.add_version();
    }


    fn update_login_time(&mut self){
        let mut map = self.get_mut_json_value();
        let mut time = Local::now();
        let jv = JsonValue::String(time.naive_local().format("%Y-%m-%dT%H:%M:%S").to_string());
        map.unwrap().insert("lastLoginTime".to_owned(), jv);
        self.add_version();
    }

    fn day_reset(&mut self) {
        self.version+=1;
    }
}

impl Dao for User{
    //获得表名
    fn get_table_name(&mut self) -> &str {
        "t_u_player"
    }
}

impl User {
    pub fn new(user_id: u32, avatar: &str, nick_name: &str) -> Self {
        let mut js_data = serde_json::map::Map::new();
        js_data.insert(USER_OL.to_string(), JsonValue::from(1));
        js_data.insert(AVATAR.to_string(), JsonValue::from(avatar));
        js_data.insert(NICK_NAME.to_string(), JsonValue::from(nick_name));
        let mut user = User::init(user_id, None,serde_json::Value::from(js_data));
        user
    }

    pub fn query(table_name:&str,user_id: u32,tem_id:Option<u32>) -> Option<Self>{
        let mut v: Vec<Value> = Vec::new();
        v.push(Value::UInt(user_id as u64));


        let mut sql = String::new();
        sql.push_str("select * from ");
        sql.push_str(table_name);
        sql.push_str(" where user_id=:user_id");
        if tem_id.is_some()
        {
            sql.push_str(" and tem_id:tem_id");
        }

        let mut q:Result<QueryResult,Error> = DB_POOL
            .exe_sql(sql.as_str(), Some(v));
        if q.is_err(){
            ()
        }
        let mut q  = q.unwrap();

        let mut data = None;
        for _qr in q {
            let (id, js) = mysql::from_row(_qr.unwrap());
            let mut u= User::init(id, tem_id,js);
            data = Some(u);
        }
        data
    }
}