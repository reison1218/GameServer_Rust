use super::*;
use crate::CONF_MAP;

pub struct DbPool {
    pub pool: Pool,
}

impl DbPool {
    ///创建一个db结构体
    pub fn new() -> DbPool {
        let str: &str = CONF_MAP.get_str("mysql");
        let pool = mysql::Pool::new(str).unwrap();
        info!("初始化dbpool完成!");
        DbPool { pool: pool }
    }

    ///执行sql
    pub fn exe_sql(
        &self,
        sql: &str,
        params: Option<Vec<Value>>,
    ) -> Result<QueryResult<'static>, Error> {
        match params {
            Some(params) => self.pool.prep_exec(sql, Params::Positional(params)),
            None => self.pool.prep_exec(sql, ()),
        }
    }
}

//fn test_postgres() {
//    let mut db_pool = Connection::connect(
//        "postgressql://root:root@localhot:3306/reison",
//        TlsMode::None,
//    )
//    .unwrap();
//    for row in &db_pool.query("select * from test", &[]).unwrap() {
//        let name: String = row.get("name");
//        println!("name: {}", name);
//    }
//}
