use super::*;
use crate::entity::contants::*;
use chrono::Local;
use log::{debug, error, info, warn, LevelFilter, Log, Record};

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: u32,    //玩家id
    pub data: JsonValue, //数据
    version: u32,        //数据版本号
}

impl Data for User {
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
    fn get_id(&self) -> Option<u32> {
        Some(self.user_id)
    }

    ///设置玩家id
    fn set_id(&mut self, id: u32) {
        self.user_id = id;
        self.add_version();
    }

    fn day_reset(&mut self) {
        unimplemented!()
    }
}

impl Dao for User {
    //获得表名
    fn get_table_name(&mut self) -> &str {
        "t_u_player"
    }

    ///查询函数
    fn query(user_id: u32, pool: &mut DbPool) -> Option<Self> {
        let mut v: Vec<Value> = Vec::new();
        v.push(Value::Int(user_id as i64));
        let mut q = pool
            .exe_sql("select * from t_u_player where user_id=:user_id", Some(v))
            .unwrap();

        let mut user: Option<User> = None;
        for _qr in q {
            let (id, js) = mysql::from_row(_qr.unwrap());
            let mut u = User::init(id, js);
            user = Some(u);
        }
        user
    }

    fn update(&mut self, pool: &mut DbPool) -> Result<u32, String> {
        let mut v: Vec<Value> = self.to_update_vec_value();
        let mut qr = pool.exe_sql(
            "update t_u_player set content=:data where user_id=:user_id",
            Some(v),
        );
        if qr.is_err() {
            let err = qr.err().unwrap();
            error!("{:?}", err);
            return Err(err.to_string());
        }
        self.clear_version();
        Ok(1)
    }

    fn insert(entity: &mut impl Dao, pool: &mut DbPool) -> Result<u32, String> {
        let mut v: Vec<Value> = entity.to_insert_vec_value();
        let mut sql = String::new();
        sql.push_str("insert into ");
        sql.push_str(entity.get_table_name());
        sql.push_str(" values(:user_id,:content)");
        let mut qr = pool.exe_sql(sql.as_str(), Some(v));

        if qr.is_err() {
            let mut str = String::from_utf8(qr.unwrap().info());
            let s = str.unwrap();
            println!("{:?}", s);
            return Err(s);
        }

        Ok(1)
    }
}

impl Entity for User {
    fn to_insert_vec_value(&mut self) -> Vec<Value> {
        let mut v: Vec<Value> = Vec::new();
        v.push(self.user_id.to_value());
        v.push(Value::from(self.data.to_string()));
        v
    }

    fn to_update_vec_value(&mut self) -> Vec<Value> {
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

    ///初始化函数
    /// 返回一个User结构体
    fn init(id: u32, js: JsonValue) -> Self {
        User {
            user_id: id,
            data: js,
            version: 0,
        }
    }
}

impl User {
    pub fn new(user_id: u32, avatar: &str, nick_name: &str) -> Self {
        let mut js_data = serde_json::map::Map::new();
        js_data.insert(USER_OL.to_string(), JsonValue::from(1));
        js_data.insert(AVATAR.to_string(), JsonValue::from(avatar));
        js_data.insert(NICK_NAME.to_string(), JsonValue::from(nick_name));
        let mut user = User::init(user_id, JsonValue::Object(js_data));
        user
    }

    pub fn update_login_time(&mut self) {
        let mut map = self.get_mut_json_value();
        let mut time = Local::now();
        let jv = JsonValue::String(time.naive_local().format("%Y-%m-%dT%H:%M:%S").to_string());
        map.unwrap().insert("lastLoginTime".to_owned(), jv);
        self.add_version();
    }
}
