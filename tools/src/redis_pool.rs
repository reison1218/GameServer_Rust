use super::*;
use redis::{Commands, Connection, FromRedisValue, Pipeline};

///redis客户端封装结构体
pub struct RedisPoolTool {
    conn: Connection,
}

///封装redis基本操作命令
impl RedisPoolTool {
    ///初始化结构体
    pub fn init(add: &str, password: &str) -> RedisPoolTool {
        let client = redis::Client::open(add).unwrap();
        info!("初始化redis客户端完成!");
        let mut redis_pool = RedisPoolTool {
            conn: client.get_connection().unwrap(),
        };
        redis::pipe()
            .cmd("AUTH")
            .arg(password)
            .execute(&mut redis_pool.conn);
        redis_pool
    }

    pub fn replace_hash<T: FromRedisValue>(
        &mut self,
        index: u32,
        hkey: &str,
        old_key: &str,
        new_key: &str,
        value: &str,
    ) -> Option<T> {
        let mut pip = get_pip();
        let conn_mut = &mut self.conn;
        pip.cmd("select").arg(index).execute(conn_mut);
        let mut res = conn_mut.hdel(hkey, old_key);
        if let Err(e) = res {
            error!("{:?}", e);
        }
        res = conn_mut.hset(hkey, new_key, value);
        pip.cmd("select").arg(0).execute(conn_mut);
        match res {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }

    ///Delete one or more keys
    pub fn del<T: FromRedisValue>(&mut self, index: u32, hkey: &str) -> Option<T> {
        let mut pip = get_pip();
        let conn_mut = &mut self.conn;
        pip.cmd("select").arg(index).execute(conn_mut);
        let res = conn_mut.del(hkey);
        pip.cmd("select").arg(0).execute(conn_mut);
        match res {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }

    ///Deletes a single (or multiple) fields from a hash
    pub fn hdel<T: FromRedisValue>(&mut self, index: u32, hkey: &str, key: &str) -> Option<T> {
        let mut pip = get_pip();
        let conn_mut = &mut self.conn;
        pip.cmd("select").arg(index).execute(conn_mut);
        let res = conn_mut.hdel(hkey, key);
        pip.cmd("select").arg(0).execute(conn_mut);
        match res {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }

    ///获得redis所有values
    pub fn hvals<T: FromRedisValue>(&mut self, index: u32, hkey: &str) -> Option<T> {
        let mut pip = get_pip();
        let conn_mut = &mut self.conn;
        pip.cmd("select").arg(index).execute(conn_mut);
        let res = conn_mut.hvals(hkey);
        pip.cmd("select").arg(0).execute(conn_mut);
        match res {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }

    ///Gets all the fields and values in a hash.
    pub fn hgetall<T: FromRedisValue>(&mut self, index: u32, hkey: &str) -> Option<T> {
        let mut pip = get_pip();
        let conn_mut = &mut self.conn;
        pip.cmd("select").arg(index).execute(conn_mut);
        let res = conn_mut.hgetall(hkey);
        pip.cmd("select").arg(0).execute(conn_mut);
        match res {
            Ok(v) => Some(v),
            Err(e) => {
                error!("{:?}", e);
                None
            }
        }
    }

    ///操作hash数据结构
    pub fn hset<T: FromRedisValue>(
        &mut self,
        index: u32,
        hkey: &str,
        key: &str,
        value: &str,
    ) -> Option<T> {
        let mut pip = get_pip();
        let conn_mut = &mut self.conn;
        pip.cmd("select").arg(index).execute(conn_mut);
        let res = conn_mut.hset(hkey, key, value);
        pip.cmd("select").arg(0).execute(conn_mut);
        match res {
            Ok(v) => Some(v),
            Err(e) => {
                error!(
                    "{:?},index:{},hkey:{:?},key:{:?},value:{:?}",
                    e, index, hkey, key, value
                );
                None
            }
        }
    }

    ///读hash数据结构
    pub fn hget<T: FromRedisValue>(&mut self, index: u32, hkey: &str, key: &str) -> Option<T> {
        let mut pip = get_pip();
        let conn_mut = &mut self.conn;
        pip.cmd("select").arg(index).execute(conn_mut);
        let res = conn_mut.hget(hkey, key);
        pip.cmd("select").arg(0).execute(conn_mut);
        if res.is_err() {
            warn!(
                "hget has error:{:?},index:{},hkey:{:?},key:{:?}",
                res.err().unwrap(),
                index,
                hkey,
                key
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
        let mut pip = get_pip();
        let conn_mut = &mut self.conn;
        pip.cmd("select").arg(index).execute(conn_mut);
        let res = conn_mut.zadd(zkey, key, value);
        pip.cmd("select").arg(0).execute(conn_mut);
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
        let mut pip = get_pip();
        let conn_mut = &mut self.conn;
        pip.cmd("select").arg(index).execute(conn_mut);
        let res = conn_mut.zrange(zkey, 0, -1);
        pip.cmd("select").arg(0).execute(conn_mut);
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
        let mut pip = get_pip();
        let conn_mut = &mut self.conn;
        pip.cmd("select").arg(index).execute(conn_mut);
        let res = conn_mut.zincr(zkey, key, "increment");
        pip.cmd("select").arg(0).execute(conn_mut);
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
        let mut pip = get_pip();
        let conn_mut = &mut self.conn;
        pip.cmd("select").arg(index).execute(conn_mut);
        let res = conn_mut.hget(hkey, key);
        pip.cmd("select").arg(0).execute(conn_mut);
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
    let value: Option<String> = rpt.replace_hash(0, "name_2_uid", "test1", "test2", "123");
    if value.is_some() {
        let value = value.unwrap();
        println!("sdfsfd:{:?}", value);
    }
}

#[test]
pub fn test() {
    test_api("redis://localhost:6379/", "reison");
}
