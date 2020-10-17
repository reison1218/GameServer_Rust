pub mod base;
pub mod battle;
pub mod protocol;
pub mod room;
pub mod server_protocol;

use log::error;
use std::path::PathBuf;

pub fn proto() {
    let res = std::env::current_dir().unwrap();
    let mut path_str = String::from(res.as_path().to_str().unwrap());
    path_str.push_str("/protos/");
    let path_buf = PathBuf::from(path_str);
    let res = std::fs::read_dir(path_buf.as_path());
    if let Err(e) = res {
        error!("{:?}", e);
        return;
    }
    let dir = res.unwrap();
    let mut files = Vec::new();
    for dir_entry in dir {
        if let Err(e) = dir_entry {
            error!("{:?}", e);
            return;
        }
        let dir_entry = dir_entry.unwrap();
        if dir_entry.file_name().eq(".DS_Store") {
            continue;
        }
        let mut proto_file = String::from("protos/");
        proto_file.push_str(dir_entry.file_name().to_str().unwrap());
        files.push(proto_file);
    }
    protoc_rust::Codegen::new()
        .out_dir("src/protos")
        .inputs(files.as_slice())
        .include("protos")
        .run()
        .expect("Running protoc failed!");
    println!("protobuf generate success!")
}
