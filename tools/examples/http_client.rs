use futures::executor::block_on;
use log::error;
use tools::http::send_http_request;

pub fn main() {
    let address = "127.0.0.1:9090";
    let path = "hello";
    let res = send_http_request(address, path, "POST", None);
    let res = block_on(res);
    match res {
        Ok(v) => {
            println!("{:?}", v);
        }
        Err(e) => {
            error!("{:?}", e);
        }
    }
}
