#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::thread_pool::ThreadIndex;

    #[test]
    fn it_works() {
        //crate::redis_pool::test_api("redis://localhost/","reison");
        // crate::protos::proto();
        // let m = || {
        //     crate::rpc_server::test_rpc_server();
        // };
        // std::thread::spawn(m);
        // std::thread::sleep(std::time::Duration::from_secs(2));
        // let time = std::time::SystemTime::now();
        // crate::rpc_client::test_rpc_client();
        // println!("{:?}", time.elapsed().unwrap());
        let pool = crate::thread_pool::ThreadWorkPool::new("test", 8 as usize);
        let m = move || {
            println!(
                "test now Thread name:{}",
                std::thread::current().name().unwrap()
            );
        };
        for i in 0..10 {
            std::thread::sleep(Duration::from_secs(1));
            pool.execute(ThreadIndex::Index(i), m);
        }
    }
}
pub mod binary;
pub mod cmd_code;
pub mod conf;
pub mod excel;
pub mod http;
pub mod json;
pub mod macros;
pub mod my_log;
pub mod net_message_io;
pub mod protos;
pub mod redis_pool;
pub mod rpc_client;
pub mod rpc_server;
pub mod tcp;
pub mod tcp_tokio;
pub mod templates;
pub mod thread_pool;
pub mod util;
pub mod ws;

use log::{error, info, warn};
use once_cell::sync::Lazy;

pub static TOKIO_RT: Lazy<tokio::runtime::Runtime> =
    Lazy::new(|| tokio::runtime::Runtime::new().unwrap());
