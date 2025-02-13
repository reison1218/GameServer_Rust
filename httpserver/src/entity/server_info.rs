use serde_json::Value;
use sqlx::{MySqlPool, Row};
use std::collections::HashMap;

#[derive(sqlx::FromRow, sqlx::Decode, sqlx::Encode, Debug, Default, Clone)]
pub struct ServerInfo {
    pub server_id: i32,
    pub name: String,
    pub ws: String,
    pub open_time: chrono::NaiveDateTime,
    pub register_state: i32,
    pub state: i32,
    pub letter: i32,
    pub target_server_id: i32,
    pub merge_times: i32,
    pub r#type: String,
    pub manager: String,
    pub inner_manager: String,
    pub server_type: i32,
}

impl ServerInfo {
    pub fn can_show(
        &self,
        acc_name: &str,
        sources: &Vec<String>,
        server_type: i32,
        ctime: i64,
    ) -> bool {
        if self.server_type != server_type {
            return false;
        }
        let read = crate::WHITE_USER_MAP.read().unwrap();
        if read.contains_key(acc_name) {
            return true;
        }
        let res = self.is_contains(&sources);
        if !res {
            return false;
        }
        if ctime < self.get_open_time() {
            return false;
        }
        true
    }

    pub fn to_json(&self) -> Value {
        let mut value = serde_json::Map::new();
        value.insert("server_id".to_owned(), Value::from(self.server_id));
        value.insert("name".to_owned(), Value::from(self.name.as_str()));
        value.insert("ws_url".to_owned(), Value::from(self.ws.as_str()));
        value.insert(
            "open_time".to_owned(),
            Value::from(self.open_time.to_string()),
        );
        value.insert(
            "register_state".to_owned(),
            Value::from(self.register_state),
        );
        value.insert("state".to_owned(), Value::from(self.state));
        value.insert("letter".to_owned(), Value::from(self.letter));
        value.insert(
            "target_server_id".to_owned(),
            Value::from(self.target_server_id),
        );
        value.insert("merge_times".to_owned(), Value::from(self.merge_times));
        value.insert("type".to_owned(), Value::from(self.r#type.as_str()));
        value.insert(
            "questionnaire_http_url".to_owned(),
            Value::from(self.server_id),
        );

        Value::from(value)
    }

    fn is_contains(&self, source: &Vec<String>) -> bool {
        let res = self.r#type.split(",");
        for s in res {
            if source.contains(&s.to_owned()) {
                return true;
            }
        }
        return false;
    }

    fn get_open_time(&self) -> i64 {
        return self.open_time.timestamp_micros();
    }
}

pub fn query_all() -> HashMap<i32, ServerInfo> {
    let mut res = HashMap::new();
    let pool: &MySqlPool = &crate::POOL;
    let a = async_std::task::block_on(async {
        let row = sqlx::query_as::<_, ServerInfo>("select * from server_list")
            .fetch_all(pool)
            .await
            .unwrap();
        row
    });
    for user in a {
        res.insert(user.server_id, user);
    }
    res
}

pub fn query(server_id: i32) -> Option<ServerInfo> {
    let pool: &MySqlPool = &crate::POOL;
    let v = async_std::task::block_on(async {
        let row = sqlx::query_as::<_, ServerInfo>("select * from server_list where server_id =?")
            .bind(server_id)
            .fetch_one(pool)
            .await;
        row
    });
    match v {
        Ok(res) => Some(res),
        Err(_) => None,
    }
}

pub fn query_merge(server_id: i32) -> Option<ServerInfo> {
    let pool: &MySqlPool = &crate::POOL;
    let db_res = async_std::task::block_on(async {
        let row = sqlx::query_as::<_, ServerInfo>(
            "SELECT * FROM server_list WHERE  server_id = (SELECT target_server_id FROM server_list WHERE server_id = ?) OR server_id = ?",
        )
        .bind(server_id)
        .bind(server_id)
        .fetch_all(pool)
        .await
        .unwrap();
        row
    });

    if db_res.is_empty() {
        return None;
    }

    for row in db_res {
        //合服过就跳过
        if row.target_server_id > 0 {
            continue;
        }
        return Some(row);
    }
    None
}

pub fn querys(servers: String) -> HashMap<i32, ServerInfo> {
    let sql = format!("select * from server_list where server_id in ({})", servers);
    let mut res = HashMap::new();
    let pool: &MySqlPool = &crate::POOL;
    let v = async_std::task::block_on(async {
        let row = sqlx::query_as::<_, ServerInfo>(sql.as_str())
            .fetch_all(pool)
            .await
            .unwrap();
        row
    });
    for user in v {
        res.insert(user.server_id, user);
    }
    res
}

pub fn query_merged_server_ids(target_id: i32) -> Vec<i32> {
    let pool: &MySqlPool = &crate::POOL;
    let rows = async_std::task::block_on(async {
        let rows = sqlx::query("select server_id from server_list where target_id = ?")
            .bind(target_id)
            .fetch_all(pool)
            .await
            .unwrap();
        rows
    });
    let mut vec = vec![];
    for row in rows {
        let id: i32 = row.get("server_id");
        vec.push(id);
    }
    vec
}

pub async fn insert(server: &ServerInfo) {
    let pool: &MySqlPool = &crate::POOL;
    let res = sqlx::query("replace into server_list(`server_id`,`name`,`ws`,`open_time`,`register_state`,`state`,`letter`,`target_server_id`,`merge_times`,`type`,`manager`,`inner_manager`,`server_type`) values(?,?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(server.server_id)
        .bind(server.name.as_str())
        .bind(server.ws.as_str())
        .bind(server.open_time)
        .bind(server.register_state)
        .bind(server.state)
        .bind(server.letter)
        .bind(server.target_server_id)
        .bind(server.merge_times)
        .bind(server.r#type.as_str())
        .bind(server.manager.as_str())
        .bind(server.inner_manager.as_str())
        .bind(server.server_type)
        .execute(pool).await;
    if let Err(e) = res {
        log::error!("{:?}", e);
    }
}
