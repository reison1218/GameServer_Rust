use super::*;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use tools::protos::base::LeaguePt;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct League {
    pub id: i8,           //段位id
    pub user_id: u32,     //玩家id
    pub name: String,     //玩家名称
    pub score: i32,       //积分
    pub rank: i32,        //排名
    pub cters: Vec<u32>,  //常用的三个角色
    pub league_time: i64, //进入段位时间
    #[serde(skip_serializing)]
    pub version: Cell<u32>, //版本号
}

unsafe impl Send for League {}

unsafe impl Sync for League {}

impl League {
    pub fn set_cters(&mut self, cters: Vec<u32>) {
        self.cters = cters;
        self.add_version();
    }

    pub fn round_reset(&mut self) {
        let old_id = self.id;
        self.id -= 1;
        if self.id <= 0 {
            self.id = 0;
            self.rank = -1;
            self.league_time = 0;
        } else {
            let res = crate::TEMPLATES
                .get_league_temp_mgr_ref()
                .get_temp(&self.id)
                .unwrap();
            if old_id != self.id {
                self.score = res.score;
                let time = chrono::Local::now();
                self.league_time = time.timestamp_millis();
            }
        }
        self.clear_version();
    }

    pub fn update_from_pt(&mut self, pt: &LeaguePt) {
        self.id = pt.league_id as i8;
        self.score = pt.league_score;
        self.league_time = pt.league_time;
        self.add_version();
    }

    pub fn into_league_pt(&self) -> LeaguePt {
        let mut lp = LeaguePt::new();
        lp.league_id = self.id as i32;
        lp.league_score = self.score;
        lp.league_time = self.league_time;
        lp
    }
}

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
    fn try_clone_for_db(&self) -> Box<dyn EntityData> {
        let res = Box::new(self.clone());
        self.version.set(0);
        res
    }
}

impl Dao for League {
    fn get_table_name(&self) -> &str {
        "t_u_league"
    }

    fn update(&self) -> Result<u32, String> {
        let mut v: Vec<Value> = Vec::new();
        v.push(self.user_id.to_value());
        let mut sql = String::new();
        sql.push_str("update ");
        sql.push_str(self.get_table_name());
        sql.push_str(self.get_update_sql().as_str());
        sql.push_str(" where user_id=:user_id ");
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
}

impl League {
    pub fn new(user_id: u32, name: String) -> Self {
        let mut l = League::default();
        l.user_id = user_id;
        l.name = name;
        l.rank = -1;
        l
    }

    pub fn get_update_sql(&self) -> String {
        let mut res = format!(
            r#" set content=JSON_REPLACE(content,"$.id",{}),content=JSON_REPLACE(content,"$.name",{:?}),content= JSON_REPLACE(content, "$.cters", JSON_ARRAY({:?})),content=JSON_REPLACE(content,"$.score",{}),content=JSON_REPLACE(content,"$.league_time",{:?})"#,
            self.id,
            self.name,
            self.cters.as_slice(),
            self.score,
            self.league_time
        );
        res = res.replace("[", "");
        res = res.replace("]", "");
        res
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
            let (_, data): (u32, serde_json::Value) = mysql::from_row(_qr.unwrap());
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
