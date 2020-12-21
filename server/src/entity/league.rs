use super::*;
use serde::{Deserialize, Serialize};
use std::cell::Cell;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct League {
    pub user_id: u32,    //玩家id
    pub name: String,    //玩家名称
    pub score: u32,      //积分
    pub rank: i32,       //排名
    pub cters: Vec<u32>, //常用的三个角色
    #[serde(skip_serializing_if = "String::is_empty")]
    pub rank_time: String, //上次离线时间
    pub version: Cell<u32>, //版本号
}

unsafe impl Send for League {}

unsafe impl Sync for League {}

impl Entity for League {
    fn set_user_id(&mut self, user_id: u32) {
        self.user_id = user_id;
    }

    fn set_ids(&mut self, user_id: u32, _: u32) {
        self.user_id = user_id;
    }

    fn update_login_time(&mut self) {}

    fn update_off_time(&mut self) {}

    fn day_reset(&mut self) {}

    fn add_version(&self) {
        let v = self.version.get() + 1;
        self.version.set(v);
    }

    fn clear_version(&self) {
        self.version.set(0);
    }

    fn get_version(&self) -> u32 {
        self.version.get()
    }

    fn get_tem_id(&self) -> Option<u32> {
        None
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

impl EntityData for League {
    fn try_clone(&self) -> Box<dyn EntityData> {
        Box::new(self.clone())
    }
}

impl Dao for League {
    fn get_table_name(&self) -> &str {
        "t_u_league"
    }
}

impl League {
    pub fn new(user_id: u32, name: String) -> Self {
        let mut l = League::default();
        l.user_id = user_id;
        l.name = name;
        l
    }

    pub fn query(table_name: &str, user_id: u32) -> Option<Self> {
        let mut v: Vec<Value> = Vec::new();
        v.push(Value::UInt(user_id as u64));

        let mut sql = String::new();
        sql.push_str("select * from ");
        sql.push_str(table_name);
        sql.push_str(" where user_id=:user_id");

        let q: Result<QueryResult, mysql::error::Error> = DB_POOL.exe_sql(sql.as_str(), Some(v));
        if q.is_err() {
            error!("{:?}", q.err().unwrap());
            return None;
        }
        let q = q.unwrap();
        let mut res = None;
        for _qr in q {
            let (_, _, data): (u32, u32, serde_json::Value) = mysql::from_row(_qr.unwrap());
            let c = League::init(data);
            res = Some(c);
            break;
        }
        if res.is_none() {
            return None;
        }
        let mut c = res.unwrap();
        c.version = Cell::new(0);
        Some(c)
    }
}
