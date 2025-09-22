mod check;
mod entity;
mod http;
mod test;

use crate::http::init_server;
use log::{error, info};
use once_cell::sync::Lazy;
use std::time::Duration;
use std::{env};
use std::sync::{Arc, RwLock};
use scheduled_thread_pool::ScheduledThreadPool;
use tools::conf::Conf;
use crate::check::{exchange_token, GoogleApiToken};

// 配置
static CONF_MAP: Lazy<Conf> = Lazy::new(|| {
    let path = env::current_dir().unwrap();
    let str = path.as_os_str().to_str().unwrap();
    let res = str.to_string() + "/server.config";
    let conf = tools::conf::read(res.as_str()).unwrap();
    info!("初始化server.config完成!");
    conf
});

// 谷歌API令牌
static SERVER_TOKEN: Lazy<Arc<RwLock<GoogleApiToken>>> = Lazy::new(|| {
    let file_path = CONF_MAP.get_str("service_account_path", "");
    let res = exchange_token(file_path.as_str()).unwrap();
    let lock = Arc::new(RwLock::new(res));
    lock
});

static TIMER:Lazy<ScheduledThreadPool> =Lazy::new(|| {
    ScheduledThreadPool::new(4)
});

pub static TOKEN_URL: Lazy<String> = Lazy::new(|| {
    let res = CONF_MAP.get_str("token_url", "");
    res
});

pub static CLIENT_ID: Lazy<String> = Lazy::new(|| {
    let res = CONF_MAP.get_str("client_id", "");
    res
});

pub static CLIENT_SECRET: Lazy<String> = Lazy::new(|| {
    let res = CONF_MAP.get_str("client_secret", "");
    res
});

pub static REDIRECT_URI: Lazy<String> = Lazy::new(|| {
    let res = CONF_MAP.get_str("redirect_url", "");
    res
});

pub static PROJECT_ID: Lazy<String> = Lazy::new(|| {
    let res = CONF_MAP.get_str("project_id", "");
    res
});

pub static PACKAGE_NAME: Lazy<String> = Lazy::new(|| {
    let res = CONF_MAP.get_str("package_name", "");
    res
});

fn init_timer() {
    let read = SERVER_TOKEN.read().unwrap();
    let expires_in = (read.expires_in - 5) as u64;
    drop(read);
    TIMER.execute_at_dynamic_rate(Duration::from_secs(expires_in), move || {
        //执行刷新操作
        info!("start refresh_google_api_token");
        let file_path = CONF_MAP.get_str("service_account_path", "");
        let res = exchange_token(file_path.as_str());

        match res {
            Ok(res) => {
                let mut write = SERVER_TOKEN.write().unwrap();
                write.expires_in = res.expires_in;
                write.access_token = res.access_token.clone();
                write.token_type = res.token_type.clone();
                let expires_in = (res.expires_in - 5) as u64;
                drop(write);
                info!("refresh_google_api_token finished!,{:?}", res);
                Some(Duration::from_secs(expires_in))
            }
            Err(e) => {
                error!("刷新 token 失败: {}", e);
                 Some(Duration::from_secs(5))
            }
        }
    });
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

fn main(){
    let time = std::time::Instant::now();
    //初始化日志
    init_log();
    //刷新谷歌api token
    init_timer();
    //初始化http服务器
    init_server();
    info!("启动成功!耗时:{}ms", time.elapsed().as_millis());
    std::thread::park();
}
