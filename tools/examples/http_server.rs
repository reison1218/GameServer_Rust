use async_std::task::block_on;
use http_types::Error;
use serde_json::Value;
use tools::http::http_server;
use tools::http::HttpServerHandler;

struct HelloHttpHandler;

impl tools::http::HttpServerHandler for HelloHttpHandler {
    fn get_path(&self) -> &str {
        "hello"
    }

    fn execute(&mut self, _params: Option<Value>) -> Result<Value, Error> {
        //todo do something
        println!("rec http request");
        let str = String::from("hello,i am http server");
        Ok(Value::from(str))
    }
}

pub fn main() {
    let mut http_vec: Vec<Box<dyn HttpServerHandler>> = Vec::new();
    http_vec.push(Box::new(HelloHttpHandler));
    let address = "127.0.0.1:9090";
    let res = block_on(http_server(address, http_vec));
    if let Err(e) = res {
        println!("{:?}", e);
    }
}
