pub mod base;
pub mod protocol;
pub mod room;

use protoc_rust::Customize;

pub fn proto() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/protos",
        input: &["protos/room.proto"],
        includes: &["protos"],
        customize: Customize {
            ..Default::default()
        },
    }).expect("protoc");
}
