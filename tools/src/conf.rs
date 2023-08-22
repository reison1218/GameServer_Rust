use crate::json::*;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;

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

    pub fn get_f64(&self, key: &str, default: f64) -> f64 {
        let value = self.conf.get(key);
        if value.is_none() {
            return default;
        }
        value.unwrap().as_f64().unwrap()
    }

    ///拿整数
    pub fn get_usize(&self, key: &str, default: usize) -> usize {
        let value = self.conf.get(key);
        if value.is_none() {
            return default;
        }
        value.unwrap().as_i64().unwrap() as usize
    }

    pub fn get_isize(&self, key: &str, default: isize) -> isize {
        let value = self.conf.get(key);
        if value.is_none() {
            return default;
        }
        value.unwrap().as_i64().unwrap() as isize
    }

    ///拿bool
    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        let value = self.conf.get(key);
        if value.is_none() {
            return default;
        }
        value.unwrap().as_bool().unwrap()
    }

    ///拿字符切片
    pub fn get_str(&self, key: &str, default: &'static str) -> String {
        let value = self.conf.get(key);
        if value.is_none() {
            return default.to_string();
        }
        let res = value.unwrap();
        let str_res = res.as_str();
        if str_res.is_none() {
            let i64_res = res.as_i64();
            match i64_res {
                Some(i64) => return i64.to_string(),
                None => return default.to_string(),
            }
        }
        str_res.unwrap().to_string()
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
