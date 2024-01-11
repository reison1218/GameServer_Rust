use serde_json::Value;
use sqlx::{MySqlPool, Row};

#[derive(sqlx::FromRow, sqlx::Decode, sqlx::Encode, Debug, Default, Clone)]
pub struct User {
    pub combine_id: i64,
    pub name: String,
    pub operator_id: i32,
    pub server_id: i32,
    pub level: i32,
    pub player_name: String,
    pub login_time: i64,
    pub json:Value,
}

impl User {
    pub fn new(
        combine_id: i64,
        name: &str,
        operator_id: i32,
        server_id: i32,
        level: i32,
        player_name: &str,
        login_time: i64,
    ) -> Self {
        let mut player = User::default();
        player.combine_id = combine_id;
        player.player_name = player_name.to_owned();
        player.name = name.to_owned();
        player.server_id = server_id;
        player.level = level;
        player.login_time = login_time;
        player.operator_id = operator_id;
        player
    }
}

pub async fn insert(user: &User) {
    let pool: &MySqlPool = &crate::POOL;
    let res = sqlx::query("replace into users(`name`,`combine_id`,`operator_id`,`server_id`,`level`,`player_name`,`login_time`) values(?,?,?,?,?,?,?)")
        .bind(user.name.as_str())
        .bind(user.combine_id)
        .bind(user.server_id)
        .bind(user.level)
        .bind(user.player_name.as_str())
        .bind(user.login_time).execute(pool).await;
    if let Err(e) = res {
        log::error!("{:?}", e);
    }
}

pub fn query_name_by_server_ids(server_ids: Vec<i32>) -> Vec<String> {
    let pool: &MySqlPool = &crate::POOL;
    let mut str = String::new();
    let mut index: usize = 0;
    let size = server_ids.len();
    for server_id in server_ids {
        str.push_str(server_id.to_string().as_str());
        if index < size - 1 {
            str.push_str(",");
        }
        index += 1;
    }

    let a = async_std::task::block_on(async {
        sqlx::query("select name from users where server_id in (?)")
            .bind(str)
            .fetch_all(pool)
            .await
            .unwrap()
    });

    let mut res = Vec::new();
    for row in a {
        let name: String = row.get(0);
        res.push(name);
    }

    res
}

pub fn find_user_login_info(acc_name: &str) -> Value {
    let pool: &MySqlPool = &crate::POOL;
    let a = async_std::task::block_on(async {
        sqlx::query_as::<_, User>("select * from users where `name` =?")
            .bind(acc_name)
            .fetch_all(pool)
            .await
            .unwrap()
    });

    let mut map = serde_json::Map::new();
    for user in a {
        let sid = user.server_id.to_string();
        let s = serde_json::json!({"login_time":user.login_time,"player_name":user.player_name.as_str(),"level":user.level});
        map.insert(sid, s);
    }
    serde_json::Value::from(map)
}
