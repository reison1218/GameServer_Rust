use super::*;

use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::str::FromStr;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Soul {
    pub user_id: u32,    //玩家id
    pub souls: Vec<u32>, //玩家拥有到灵魂
    #[serde(skip_serializing)]
    pub version: Cell<u32>, //版本号
}

unsafe impl Send for Soul {}

unsafe impl Sync for Soul {}

impl Entity for Soul {
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

impl EntityData for Soul {
    fn try_clone_for_db(&self) -> Box<dyn EntityData> {
        let res = Box::new(self.clone());
        self.version.set(0);
        res
    }
}

impl Dao for Soul {
    fn get_table_name(&self) -> &str {
        "t_u_soul"
    }
}

impl Soul {
    pub fn new(user_id: u32) -> Self {
        let mut gf = Soul::default();
        let default_soul = crate::TEMPLATES
            .get_constant_temp_mgr_ref()
            .temps
            .get("default_soul")
            .unwrap();
        let id = u32::from_str(default_soul.value.as_str());
        let gf_id;
        if let Err(e) = id {
            error!("{:?}", e);
            gf_id = 1;
        } else {
            gf_id = id.unwrap();
        }
        gf.user_id = user_id;
        gf.souls.push(gf_id);
        gf
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
            let c = Soul::init(data);
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
