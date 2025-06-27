mod http;
mod ws;

use crate::http::http_server;
use crate::ws::ws::{get_zones, handle_port};
use lazy_static::lazy_static;
use log::error;
use scheduled_thread_pool::ScheduledThreadPool;
use std::env;
use tokio::time::{Duration, sleep};
use tools::conf::Conf;

use std::error::Error;

lazy_static! {
        ///配置文件
    static ref CONF_MAP : Conf = {
        let path = env::current_dir().unwrap();
        let str = path.as_os_str().to_str().unwrap();
        let res = str.to_string()+"/server.config";
        let conf = tools::conf::read(res.as_str()).unwrap();
        log::info!("初始化server.config完成!");
        conf
    };

    static ref TIMER:ScheduledThreadPool = ScheduledThreadPool::new(2);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let time = std::time::Instant::now();
    init_log();
    init_http_server();
    init_ws();
    log::info!("启动成功!耗时:{}ms", time.elapsed().as_millis());
    std::thread::park();
    Ok(())
}

pub fn init_ws() {
    let m = async {
        let mut run_zones = vec![];
        loop {
            let zones = get_zones().await;
            if zones.is_none() {
                continue;
            }
            let zones = zones.unwrap();
            for (_, port) in zones {
                if run_zones.contains(&port) {
                    continue;
                }
                tokio::spawn(async move {
                    if let Err(e) = handle_port(port).await {
                        error!("Error on port {}: {}", port, e);
                    }
                });
                run_zones.push(port);
            }
            sleep(Duration::from_secs(60)).await;
        }
    };
    tokio::spawn(m);
}

///
/// 初始化http服务
pub fn init_http_server() {
    http_server::init_server();
}

///
/// 初始化日志
pub fn init_log() {
    let path = env::current_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_owned()
        + "/log_config.yaml";
    tools::my_log::init_log(path.as_str());
}
