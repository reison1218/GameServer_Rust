use log::info;
use std::time;

///初始化日志
///传入日志配置文件（yaml）
pub fn init_log(config_path: &str) {
    let log_time = time::SystemTime::now();
    log4rs::init_file(config_path, Default::default()).unwrap();
    info!(
        "日志模块初始化完成！耗时:{}ms",
        log_time.elapsed().unwrap().as_millis()
    );
}
