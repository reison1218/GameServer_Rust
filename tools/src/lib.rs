#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        //crate::redis_pool::test_api("redis://localhost/","reison");
        crate::protos::proto();
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
pub mod tcp;
pub mod thread_pool;
pub mod util;
use log::{error, info, warn};
