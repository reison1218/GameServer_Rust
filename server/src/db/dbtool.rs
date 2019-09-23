use super::*;

pub struct DbPool {
    pub pool: Pool,
}

impl DbPool {
    ///创建一个db结构体
    pub fn new() -> DbPool {
        let mut pool = mysql::Pool::new("mysql://root:root@localhost:3306/reison").unwrap();
        DbPool { pool: pool }
    }

    ///执行sql
    pub fn exe_sql(
        &mut self,
        sql: &str,
        params: Option<Vec<Value>>,
    ) -> Result<QueryResult<'static>, Error> {
        if params.is_some() {
            self.pool
                .prep_exec(sql, Params::Positional(params.unwrap()))
        } else {
            self.pool.prep_exec(sql, ())
        }
    }
}

pub struct TestDb {
    id: u32,
    name: String,
    time: chrono::NaiveDateTime,
}

#[test]
fn main() {
    test_mysql();
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

pub fn test_mysql() {
    info!("创建连接mysql");
    let mut pool = DbPool::new();
    let mut qr = pool.exe_sql("select * from test", None).unwrap();
    for _qr in qr {
        let (name, create_time, id) = mysql::from_row(_qr.unwrap());
        let obtl = TestDb {
            id: id,
            name: name,
            time: create_time,
        };
        info!("mysql拿出的时间：{:?}", obtl.time);
    }

    let id = 123;
    let name = "sdfsdf".to_string();

    let d = NaiveDate::from_ymd(2015, 6, 3);
    let t = NaiveTime::from_hms_milli(12, 34, 56, 789);
    let time = chrono::NaiveDateTime::new(d, t);
    let mut str = "update test set name='aaaaa',create_time=:create_time where id=1";
    let mut v: Vec<Value> = Vec::new();
    v.push(time.to_string().as_str().to_value());

    pool.exe_sql(str, Some(v));

    str = "insert into test(id,name,create_time) values(:id,:name,:create_time)";
    let mut v: Vec<Value> = Vec::new();
    v.push(Value::Int(3));
    let _str = "mysql".to_string();
    v.push(_str.to_value());
    let local: DateTime<Local> = Local::now();
    v.push(local.naive_local().to_value());
    let mut re = pool.exe_sql(str, Some(v));
    if re.is_err() {
        println!("{:?}", re.err().unwrap().to_string());
    }
}
