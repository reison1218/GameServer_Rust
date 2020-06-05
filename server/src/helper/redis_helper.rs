use super::*;
use futures::AsyncWriteExt;
use serde_json::Value;
use std::str::FromStr;

///修改玩家redis状态
pub fn modify_redis_user(user_id: u32, key: String, value: Value) {
    let mut redis_write = REDIS_POOL.write().unwrap();
    let res: Option<String> = redis_write.hget(1, "uid_2_pid", user_id.to_string().as_str());
    if res.is_none() {
        return;
    }
    let pid = res.unwrap();

    let res: Option<String> = redis_write.hget(0, "users", &pid.to_string().as_str());
    if res.is_none() {
        return;
    }
    let res = res.unwrap();
    let json = Value::from_str(res.as_str());

    match json {
        Ok(mut json_value) => {
            let json_res = json_value.as_object_mut();
            if json_res.is_some() {
                json_res.unwrap().insert(key.to_owned(), value);

                let res: Option<u32> = redis_write.hset(
                    0,
                    "users",
                    pid.to_string().as_str(),
                    json_value.to_string().as_str(),
                );
            }
        }
        Err(e) => {
            error!("{:?}", e);
        }
    }
}

///从redis获得玩家数据
pub fn get_user_from_redis(user_id: u32) -> Option<Value> {
    let mut redis_write = REDIS_POOL.write().unwrap();
    let value: Option<String> = redis_write.hget(1, "uid_2_pid", user_id.to_string().trim());
    if value.is_none() {
        return None;
    }
    let pid = value.unwrap();

    let res: Option<String> = redis_write.hget(0, "users", &pid.to_string().as_str());
    if res.is_none() {
        return None;
    }
    let res = res.unwrap();

    let json = Value::from_str(res.as_str());

    match json {
        Ok(json_value) => Some(json_value),
        Err(e) => {
            error!("{:?}", e);
            None
        }
    }
}
