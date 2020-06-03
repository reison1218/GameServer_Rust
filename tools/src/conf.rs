use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct Conf {
    conf: HashMap<String, Value>,
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

    ///拿字符切片
    pub fn get_str(&self, key: &str) -> &str {
        let value = self.conf.get(key);
        if value.is_none() {
            return "";
        }
        value.unwrap().as_str().unwrap()
    }
}

///读取配置文件
fn read_conf_from_file<P: AsRef<Path>>(path: P) -> Result<HashMap<String, Value>, Box<dyn Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `map`.
    let map = serde_json::from_reader(reader)?;

    // Return the `map`.
    Ok(map)
}
