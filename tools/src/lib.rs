#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        //crate::redis_pool::test_api("redis://localhost/","reison");
        crate::protos::proto();
        // let m = || {
        //     crate::rpc_server::test_rpc_server();
        // };
        // std::thread::spawn(m);
        // std::thread::sleep(std::time::Duration::from_secs(2));
        // let time = std::time::SystemTime::now();
        // crate::rpc_client::test_rpc_client();
        // println!("{:?}", time.elapsed().unwrap());
    }
}
pub mod binary;
pub mod cmd_code;
pub mod conf;
pub mod http;
pub mod macros;
pub mod my_log;
pub mod protos;
pub mod redis_pool;
pub mod rpc_client;
pub mod rpc_server;
pub mod tcp;
pub mod templates;
pub mod thread_pool;
pub mod util;
use log::{error, info, warn};
