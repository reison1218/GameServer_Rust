use protoc_rust::Customize;

pub fn proto() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/protos",
        input: &["protos/protocol.proto"],
        includes: &["protos"],
        customize: Customize {
            ..Default::default()
        },
    })
    .expect("protoc");
}
