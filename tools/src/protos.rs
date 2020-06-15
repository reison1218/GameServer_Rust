use protoc_rust::Customize;

pub mod base;
pub mod protocol;
pub mod room;
pub mod server_protocol;

pub fn proto() {
    protoc_rust::Codegen::new()
             .out_dir("src/protos")
             .inputs(&["protos/base.proto", "protos/protocol.proto","protos/room.proto","protos/server_protocol.proto"])
             .include("protos")
             .run()
             .expect("Running protoc failed!");
    println!("protobuf generate success!")
}
