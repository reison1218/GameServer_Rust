use std::collections::HashMap;
use serde_json::value::Value;
use tera::{Result};
use crate::caches::video_tags::VIDEO_TAGS as LIST;

/// 得到标签名称
pub fn tag_name<'r, 's>(val: &'r Value, _data: &'s HashMap<String, Value>) -> Result<Value> { 
    if let Value::Number(n)  = val { 
        let id = n.as_u64().unwrap() as usize;
        if id == 0 { 
            return Ok(json!(""));
        }
        let roles = LIST.lock().unwrap();
        if let Some(v) = roles.get(&id) { 
            return Ok(json!(v));
        }
    }
    Ok(json!("错误!!!"))
}
