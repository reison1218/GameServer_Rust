use std::collections::HashMap;

use sqlx::{MySqlPool, Row};

#[derive(sqlx::FromRow, sqlx::Decode, sqlx::Encode, Debug, Default, Clone)]
pub struct WxUsersSubscribe {
    pub name: String,
    pub open_id: String,
    pub templ_ids: HashMap<String, i32>,
}

impl WxUsersSubscribe {
    pub fn add_templ_id(&mut self, id: &str) {
        let mut v = 1;
        if self.templ_ids.contains_key(id) {
            v = *self.templ_ids.get(id).unwrap() + 1;
        }
        self.templ_ids.insert(id.to_string(), v);
    }

    pub fn set_templ_ids(&mut self, str: String) {
        if str.is_empty() {
            return;
        }
        let strs = str.split(",");
        for s in strs {
            let ss: Vec<&str> = s.split("|").collect();
            self.templ_ids.insert(
                ss.get(0).unwrap().to_string(),
                ss.get(1).unwrap().parse::<i32>().unwrap(),
            );
        }
    }

    pub fn get_templ_id_map_str(&self) -> String {
        let mut sb = String::new();
        let mut index = 0;
        let size = self.templ_ids.len();
        self.templ_ids.iter().for_each(|(id, times)| {
            sb.push_str(id);
            sb.push_str("|");
            sb.push_str(times.to_string().as_str());
            if index < size - 1 {
                sb.push_str(",");
            }
            index += 1;
        });
        sb
    }
}

pub async fn insert(wx: &WxUsersSubscribe) {
    let pool: &MySqlPool = &crate::POOL;
    let res =
        sqlx::query("replace into wx_users_subscribe(`name`,`open_id`,`templ_ids`) values(?,?,?)")
            .bind(wx.name.as_str())
            .bind(wx.open_id.as_str())
            .bind(wx.get_templ_id_map_str())
            .execute(pool)
            .await;
    if let Err(e) = res {
        log::error!("{:?}", e);
    }
}

pub fn querys_by_names(names:String) -> HashMap<String,WxUsersSubscribe> {
    let pool: &MySqlPool = &crate::POOL;

    let a = async_std::task::block_on(async {
        sqlx::query("select * from wx_users_subscribe where name in(?)")
        .bind(names)
            .fetch_all(pool)
            .await
            .unwrap()
    });

    let mut res = HashMap::new();
    for row in a {
        let name: String = row.get(0);
        let open_id: String = row.get(1);
        let templ_ids: String = row.get(2);
        let mut wx = WxUsersSubscribe::default();
        wx.open_id = open_id;
        wx.set_templ_ids(templ_ids);
        wx.name = name.clone();
        res.insert(name,wx);
    }
    res
}
