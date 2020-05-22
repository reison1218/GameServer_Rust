pub mod base;
pub mod protocol;
pub mod room;

pub fn proto() {
    protoc_rust::Codegen::new()
             .out_dir("src/protos")
             .inputs(&["protos/base.proto", "protos/protocol.proto","protos/room.proto"])
             .include("protos")
             .run()
             .expect("Running protoc failed!");
}
