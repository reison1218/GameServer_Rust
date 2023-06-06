use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

use crate::JsonValue;

///conf of struct
#[derive(Default)]
pub struct Conf {
    pub conf: HashMap<String, JsonValue>,
}

impl Conf {
    ///初始化配置文件
    pub fn init(path: &str) -> Conf {
        let conf = read_conf_from_file(path);
        let conf = Conf {
            conf: conf.unwrap(),
        };
        conf
    }

    ///拿整数
    pub fn get_usize(&self, key: &str) -> usize {
        let value = self.conf.get(key);
        if value.is_none() {
            return 0;
        }
        value.unwrap().as_i64().unwrap() as usize
    }

    ///拿bool
    pub fn get_bool(&self, key: &str) -> bool {
        let value = self.conf.get(key);
        if value.is_none() {
            return false;
        }
        value.unwrap().as_bool().unwrap()
    }

    ///拿字符切片
    pub fn get_str(&self, key: &str) -> &str {
        let value = self.conf.get(key);
        if value.is_none() {
            return "";
        }
        let res = value.unwrap();
        res.as_str().unwrap()
    }

    pub fn new(map: HashMap<String, JsonValue>) -> Self {
        let mut conf = Conf::default();
        conf.conf = map;
        conf
    }
}

///读取配置文件
fn read_conf_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, JsonValue>, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `map`.
    let map = serde_json::from_reader(reader)?;

    // Return the `map`.
    Ok(map)
}

pub fn read(path: &str) -> anyhow::Result<Conf> {
    let file = File::open(path).unwrap();
    let buf_reader = BufReader::new(file);
    let mut map = HashMap::new();
    for line in buf_reader.lines() {
        let a = line?;

        if a.starts_with("#") {
            continue;
        }
        if a.is_empty() {
            continue;
        }
        let v: Vec<&str> = a.split("=").into_iter().map(|x| x.trim()).collect();
        let key = v.get(0).unwrap().to_owned().to_owned();
        let value = serde_json::Value::from_str(v.get(1).unwrap().to_owned());
        match value {
            Ok(v) => {
                map.insert(key, v);
            }
            Err(_) => {
                let v = serde_json::Value::from(v.get(1).unwrap().to_owned());
                map.insert(key, v);
            }
        }
    }
    Ok(Conf::new(map))
}
