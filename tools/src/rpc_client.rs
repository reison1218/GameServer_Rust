use crate::protos::rpc::greeter_client::GreeterClient;
use crate::protos::rpc::HelloRequest;

pub fn test_rpc_client() {
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_io();
    let res = builder.build().unwrap();
    let _ = res.block_on(start_client());
}

async fn start_client() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.test(request).await?;

    println!("RESPONSE={:?}", response.get_ref().message);
    Ok(())
}
