use crate::{JsonValue, REDIS_POOL};
use crate::{REDIS_INDEX_USERS, REDIS_KEY_UID_2_PID, REDIS_KEY_USERS};
use log::error;
use std::str::FromStr;

///从redis获得玩家数据
pub fn get_user_from_redis(user_id: u32) -> Option<JsonValue> {
    let mut redis_lock = REDIS_POOL.lock().unwrap();
    let value: Option<String> = redis_lock.hget(
        REDIS_INDEX_USERS,
        REDIS_KEY_UID_2_PID,
        user_id.to_string().trim(),
    );
    if value.is_none() {
        return None;
    }
    let pid = value.unwrap();

    let res: Option<String> = redis_lock.hget(
        REDIS_INDEX_USERS,
        REDIS_KEY_USERS,
        &pid.to_string().as_str(),
    );
    if res.is_none() {
        return None;
    }
    let res = res.unwrap();

    let json = JsonValue::from_str(res.as_str());

    match json {
        Ok(json_value) => Some(json_value),
        Err(e) => {
            error!("{:?}", e);
            None
        }
    }
}
