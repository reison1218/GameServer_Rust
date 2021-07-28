use log::{info, LevelFilter};
use simplelog::{CombinedLogger, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;
use std::time;

fn setup() {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install().unwrap();
}

///init the log
///info_path:the info log path
///error_path:the error log path
pub fn init_log(info_path: &str, error_path: &str) {
    setup();
    let log_time = time::SystemTime::now();
    let mut config = simplelog::ConfigBuilder::new();
    config.set_time_format_str("%Y-%m-%d %H:%M:%S");
    config.set_time_to_local(true);
    config.set_target_level(LevelFilter::Error);
    config.set_location_level(LevelFilter::Error);
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, config.build(), TerminalMode::Mixed),
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
        "log model init finish!take time:{}ms",
        log_time.elapsed().unwrap().as_millis()
    );
}
