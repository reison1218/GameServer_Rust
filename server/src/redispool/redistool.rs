use super::*;
use crate::CONF_MAP;

///redis客户端封装结构体
pub struct RedisPoolTool {
    client: Client,
    conn: Connection,
}

///封装redis基本操作命令
impl RedisPoolTool {
    ///初始化结构体
    pub fn init() -> RedisPoolTool {
        let str: &str = CONF_MAP.get_str("redis");
        let mut client = redis::Client::open(str).unwrap();
        info!("初始化redis客户端完成!");
        RedisPoolTool {
            client: client.clone(),
            conn: client.get_connection().unwrap(),
        }
    }

    ///操作hash数据结构
    pub fn set_hset(&mut self, hkey: &str, key: &str, value: &str) -> RedisResult<Value> {
        get_pip()
            .cmd("HSET")
            .arg(hkey)
            .arg(key)
            .arg(value)
            .query(&mut self.conn)
    }

    ///读hash数据结构
    pub fn get_hset(&mut self, hkey: &str, key: &str) -> RedisResult<Value> {
        get_pip()
            .cmd("HGET")
            .arg(hkey)
            .arg(key)
            .query(&mut self.conn)
    }

    ///操作有序集合，单个添加
    /// zkey:有序集合的key
    /// key：有序集合成员的key
    /// value：有序集合成员value
    pub fn add_zset(&mut self, zkey: &str, key: &str, value: isize) -> RedisResult<Value> {
        get_pip()
            .cmd("ZADD")
            .arg(zkey)
            .arg(value)
            .arg(key)
            .query(&mut self.conn)
    }

    ///得到整个有序集合
    /// zkey:有序集合的key
    pub fn get_zset(&mut self, zkey: &str) -> RedisResult<Value> {
        get_pip()
            .cmd("ZRANGE")
            .arg(zkey)
            .arg(0)
            .arg(-1)
            .arg("WITHSCORES")
            .query(&mut self.conn)
    }

    ///有序集合自增操作
    /// zkey:有序集合的key
    /// key：有序集合成员的key
    pub fn zincrby(&mut self, zkey: &str, key: &str) -> RedisResult<Value> {
        get_pip()
            .cmd("ZINCRBY")
            .arg(zkey)
            .arg("increment")
            .query(&mut self.conn)
    }
}

pub fn get_pip() -> Pipeline {
    redis::pipe()
}

#[test]
pub fn test_redis() {
    // connect to redis
    let client = redis::Client::open("redis://127.0.0.1:6379/").unwrap();
    let mut con = client.get_connection().unwrap();
    let mut pip = redis::pipe();

    let mut r: Value = pip.cmd("SELECT").arg("0").query(&mut con).unwrap();

    r = pip
        .cmd("HSET")
        .arg("test")
        .arg("key_1")
        .arg("value_1")
        .query(&mut con)
        .unwrap();
}

pub fn test_api() {
    let mut rpt = RedisPoolTool::init();
    let mut value = rpt.add_zset("ztest", "ztest", 1);
    if value.is_err() {
        println!("{:?}", value.err().unwrap());
        return;
    }
    value = rpt.get_zset("ztest");
    if value.is_err() {
        println!("{:?}", value.err().unwrap())
    } else {
        println!("{:?}", value.unwrap());
    }
}
