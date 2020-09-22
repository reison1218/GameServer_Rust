use std::collections::HashMap;
use serde_json::value::Value;
use tera::{Result};
use crate::caches::STATES;

/// 状态名称
pub fn state_name<'r, 's>(val: &'r Value, _data: &'s HashMap<String, Value>) -> Result<Value> { 
    if let Value::Number(v) = val { 
        let n = v.as_u64().unwrap();
        if n != 0 && n != 1 { 
            return Ok(json!("未知等级"));
        }
        
        return Ok(json!(STATES[n as usize]));
    }
    Ok(json!("错误!!!"))
}

pub fn yes_no<'r, 's>(val: &'r Value, _data: &'s HashMap<String, Value>) -> Result<Value> { 
    if let Value::Number(v) = val { 
        let n = v.as_u64().unwrap();
        if n == 1 { 
            return Ok(json!("是"));
        }
        if n == 0 { 
            return Ok(json!("否"));
        }
    }
    Ok(json!("错误!!!"))
}

pub mod menus;
pub mod admin_roles;
pub mod ads;
pub mod video_tags;
pub mod video_authors;
