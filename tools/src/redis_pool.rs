use super::*;
use redis::{Client, Commands, Connection, FromRedisValue, Pipeline};

///redis客户端封装结构体
pub struct RedisPoolTool {
    client: Client,
    conn: Connection,
}

///封装redis基本操作命令
impl RedisPoolTool {
    ///初始化结构体
    pub fn init(add: &str, password: &str) -> RedisPoolTool {
        let client = redis::Client::open(add).unwrap();
        info!("初始化redis客户端完成!");
        let mut redis_pool = RedisPoolTool {
            client: client.clone(),
            conn: client.get_connection().unwrap(),
        };
        redis::pipe()
            .cmd("AUTH")
            .arg(password)
            .execute(&mut redis_pool.conn);
        redis_pool
    }

    ///操作hash数据结构
    pub fn hset<T: FromRedisValue>(
        &mut self,
        index: u32,
        hkey: &str,
        key: &str,
        value: &str,
    ) -> Option<T> {
        get_pip().cmd("select").arg(index).execute(&mut self.conn);
        let res = self.conn.hset(hkey, key, value);
        get_pip().cmd("select").arg(0).execute(&mut self.conn);
        match res {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }

    ///读hash数据结构
    pub fn hget<T: FromRedisValue>(&mut self, index: u32, hkey: &str, key: &str) -> Option<T> {
        get_pip().cmd("select").arg(index).execute(&mut self.conn);
        let res = self.conn.hget(hkey, key);
        get_pip().cmd("select").arg(0).execute(&mut self.conn);
        if res.is_err() {
            error!(
                "hget has error:{:?},index:{},key:{:?}",
                res.err().unwrap(),
                index,
                hkey
            );
            return None;
        }
        Some(res.unwrap())
    }

    ///操作有序集合，单个添加
    /// zkey:有序集合的key
    /// key：有序集合成员的key
    /// value：有序集合成员value
    pub fn zadd<T: FromRedisValue>(
        &mut self,
        index: u32,
        zkey: &str,
        key: &str,
        value: isize,
    ) -> Option<T> {
        get_pip().cmd("select").arg(index).execute(&mut self.conn);
        let res = self.conn.zadd(zkey, key, value);
        get_pip().cmd("select").arg(0).execute(&mut self.conn);
        match res {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }

    ///得到整个有序集合
    /// zkey:有序集合的key
    pub fn zrange<T: FromRedisValue>(&mut self, index: u32, zkey: &str) -> Option<T> {
        get_pip().cmd("select").arg(index).execute(&mut self.conn);
        let res = self.conn.zrange(zkey, 0, -1);
        get_pip().cmd("select").arg(0).execute(&mut self.conn);
        match res {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }

    ///有序集合自增操作
    /// zkey:有序集合的key
    /// key：有序集合成员的key
    pub fn zincrby<T: FromRedisValue>(&mut self, index: u32, zkey: &str, key: &str) -> Option<T> {
        get_pip().cmd("select").arg(index).execute(&mut self.conn);
        let res = self.conn.zincr(zkey, key, "increment");
        get_pip().cmd("select").arg(0).execute(&mut self.conn);
        match res {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }

    ///读hash数据结构
    pub fn test<T: FromRedisValue>(&mut self, index: u32, hkey: &str, key: &str) -> Option<T> {
        get_pip().cmd("select").arg(index).execute(&mut self.conn);
        let res = self.conn.hget(hkey, key);
        get_pip().cmd("select").arg(0).execute(&mut self.conn);
        if res.is_err() {
            error!("{:?}", res.err().unwrap());
            return None;
        }
        Some(res.unwrap())
    }
}

pub fn get_pip() -> Pipeline {
    redis::pipe()
}

pub fn test_api(add: &str, pass: &str) {
    let mut rpt = RedisPoolTool::init(add, pass);
    let value: Option<Vec<u32>> = rpt.hget(1, "uid_2_pid", "1011000001");
    if value.is_some() {
        let value = value.unwrap();
        println!("sdfsfd:{:?}", value);
    }
}
