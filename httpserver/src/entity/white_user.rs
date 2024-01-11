use sqlx::MySqlPool;

#[derive(sqlx::FromRow, sqlx::Decode, sqlx::Encode, Debug, Default, Clone)]
pub struct WhiteUserInfo {
    pub name: String,
}
pub fn query_all() -> Vec<WhiteUserInfo> {
    let pool: &MySqlPool = &crate::POOL;

    let res = async_std::task::block_on(async {
        sqlx::query_as::<_, WhiteUserInfo>("select * from white_users")
            .fetch_all(pool)
            .await
            .unwrap()
    });
    res
}

pub fn insert(name: &str) {
    let pool: &MySqlPool = &crate::POOL;

    async_std::task::block_on(async {
        sqlx::query("replace into white_users(name) value(?)")
            .bind(name)
            .execute(pool)
            .await
            .unwrap()
    });
}

pub fn delete(name: &str) {
    let pool: &MySqlPool = &crate::POOL;

    async_std::task::block_on(async {
        sqlx::query("delete from white_users where name = ?")
            .bind(name)
            .execute(pool)
            .await
            .unwrap()
    });
}
