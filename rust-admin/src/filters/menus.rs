use std::collections::HashMap;
use serde_json::value::Value;
use tera::{Result};
use crate::caches::menus::{MAIN_MENUS};

/// 得到菜单名称
pub fn menu_name<'r, 's>(val: &'r Value, _data: &'s HashMap<String, Value>) -> Result<Value> { 
    if let Value::Number(n)  = val { 
        let id = n.as_u64().unwrap() as usize;
        if id == 0 { 
            return Ok(json!(""));
        }
        let menus = &*MAIN_MENUS.lock().unwrap();
        if let Some(v) = menus.get(&id) { 
            return Ok(json!(v));
        }
    }
    Ok(json!("错误!!!"))
}
