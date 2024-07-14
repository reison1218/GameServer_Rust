use log::{error, info, warn};

pub fn main() {
    log_init();
    info!("test");

    warn!("test");

    error!("test");
}

pub fn log_init(){
    log4rs::init_file("config/log_config.yaml", Default::default()).unwrap();
}