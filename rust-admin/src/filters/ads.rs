use std::collections::HashMap;
use serde_json::value::Value;
use tera::{Result};
use crate::caches::ads::POSITIONS;

/// 页面位置
pub fn position_name<'r, 's>(val: &'r Value, _data: &'s HashMap<String, Value>) -> Result<Value> { 
    if let Value::Number(v) = val { 
        let n = v.as_u64().unwrap() as usize;
        if let Some(name) = POSITIONS.get(&n) { 
            return Ok(json!(name));
        }
    }
    Ok(json!("错误!!!"))
}
