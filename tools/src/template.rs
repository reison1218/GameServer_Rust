use serde_json::{Value, Map};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::path::Path;
use std::borrow::Borrow;

///读取配置文件
pub fn read_templates_from_dir<P: AsRef<Path>>(path: P) -> Result<HashMap<String, Vec<Map<String,Value>>>, Box<Error>> {
    // Open the file in read-only mode with buffer.
    let result = std::fs::read_dir(path)?;

    let mut file_map:HashMap<String, Vec<Map<String,Value>>> = HashMap::new();
    for f in result{
        let file= f.unwrap();
        let name = file.file_name();
        let mut str = String::new();
        str.push_str(file.path().parent().unwrap().to_str().unwrap().borrow());
        str.push_str("/");
        str.push_str(name.to_str().unwrap());
        let mut file = File::open(str).unwrap();
        let mut reader = BufReader::new(file);
        let mut string = String::new();
        reader.read_line(&mut string);
        let map:serde_json::Result<Vec<Map<String,Value>>> = serde_json::from_str(string.as_ref());
        let map = map.unwrap();
        let mut name = name.to_str().unwrap().to_string();
        let beta_offset = name.find('.').unwrap_or(name.len());
        name.replace_range(beta_offset.., "");
        file_map.insert(name,map);
    }
    Ok(file_map)
}