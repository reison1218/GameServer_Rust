use log::{info, LevelFilter};
use simplelog::{CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;
use std::time;

///初始化日志
/// 传入info_path作为 info文件路径
/// 传入error_path作为 error文件路径
pub fn init_log(info_path: &str, error_path: &str) {
    let log_time = time::SystemTime::now();
    let mut config = simplelog::ConfigBuilder::new();
    config.set_time_format_str("%Y-%m-%d %H:%M:%S");
    config.set_time_to_local(true);
    config.set_target_level(LevelFilter::Error);
    config.set_location_level(LevelFilter::Error);
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, config.build(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            LevelFilter::Info,
            config.build(),
            File::create(info_path).unwrap(),
        ),
        WriteLogger::new(
            LevelFilter::Error,
            config.build(),
            File::create(error_path).unwrap(),
        ),
    ])
    .unwrap();
    info!(
        "日志模块初始化完成！耗时{}ms",
        log_time.elapsed().unwrap().as_millis()
    );
}
