use tonic::{transport::Server, Request, Response, Status};

use crate::protos::rpc::greeter_server::{Greeter, GreeterServer};
use crate::protos::rpc::{HelloReply, HelloRequest};

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }
}

pub fn test_rpc_server() {
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_io();
    let mut res = builder.build().unwrap();
    res.block_on(start_server());
}

async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
