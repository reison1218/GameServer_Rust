use super::*;
use crate::template::template_contants::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct Template {
    pub id: u64,
    pub value: JsonValue,
}

pub struct Templates {
    pub temps: HashMap<String, HashMap<u64, Template>>,
}

impl Templates {
    pub fn new(path: &str) -> Templates {
        let mut map = HashMap::new();
        let res_map = tools::template::read_templates_from_dir(path);
        let res_map = res_map.unwrap();
        for json in res_map {
            let name = json.0;
            let v = json.1;
            let mut _map = HashMap::new();
            if name.eq("") {
            } else {
                for j in v {
                    let value = JsonValue::from(j);
                    let id = value.get("ID").unwrap().as_u64().unwrap();
                    _map.insert(id, Template { id, value });
                }
                map.insert(name, _map);
            }
        }
        Templates { temps: map }
    }

    ///根据配置表名字和id获得配置
    pub fn get(&self, name: &str, id: &u64) -> Option<&Template> {
        let res = self.temps.get(name);
        if res.is_none() {
            return None;
        }
        let res = res.unwrap();
        let res = res.get(id);
        if res.is_none() {
            return None;
        }
        Some(res.unwrap())
    }
}
