use async_std::task::block_on;
use http_types::Error;
use serde_json::Value;
use tools::http::http_server;
use tools::http::HttpServerHandler;

#[derive(Default)]
struct HelloHttpHandler {}

impl tools::http::HttpServerHandler for HelloHttpHandler {
    fn get_path(&self) -> &str {
        "hello"
    }

    fn execute(&mut self, params: Option<Value>) -> Result<Value, Error> {
        //todo do something
        println!("rec http request");
        let str = String::from("hello,i am http server");
        Ok(Value::from(str))
    }
}

pub fn main() {
    let mut http_vec: Vec<Box<dyn HttpServerHandler>> = Vec::new();
    http_vec.push(Box::new(HelloHttpHandler::default()));
    let address = "127.0.0.1:9090";
    block_on(http_server(address, http_vec));
}
