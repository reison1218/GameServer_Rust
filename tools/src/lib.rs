#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        crate::protos::proto();
    }
}
pub mod tcp;
pub mod util;
pub mod thread_pool;
pub mod conf;
pub mod http;
pub mod my_log;
pub mod cmd_code;
pub mod protos;
pub mod template;
pub mod binary;
use log::{error, info, LevelFilter};
