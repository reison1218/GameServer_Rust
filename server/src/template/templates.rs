use super::*;
use std::collections::HashMap;
use crate::template::template_contants::*;
use serde_json::Value as JsonValue;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Template{
    id: u64,
    value:JsonValue,
}

pub struct Templates{
    pub temps:HashMap<String,HashMap<u64,Template>>,
}

impl Templates{
    pub fn new(path:&str)->Templates{
        let mut map = HashMap::new();
        let res_map = tools::template::read_templates_from_dir(path);
        let res_map = res_map.unwrap();
        for json in res_map{
            let name = json.0;
            let v = json.1;
            let mut _map = HashMap::new();
            if name.eq(""){

            }else{
                for j in v{
                    let value = JsonValue::from(j);
                    let id = value.get("id").unwrap().as_u64().unwrap();
                    _map.insert(id,Template{id ,value});
                }
                map.insert(name,_map);
            }
        }
        Templates{temps:map}
    }
}