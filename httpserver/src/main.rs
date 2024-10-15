pub mod db;
pub mod entity;
pub mod handler;
pub mod http_server;
pub mod wx;
use std::env;

use lazy_static::lazy_static;
use sqlx::MySqlPool;
use tools::conf::Conf;

lazy_static! {
        ///配置文件
        static ref CONF_MAP : Conf = {
            let path = std::env::current_dir().unwrap();
            let str = path.as_os_str().to_str().unwrap();
            let res = str.to_string()+"/server.config";
            let conf = tools::conf::read(res.as_str()).unwrap();
            log::info!("初始化server.config完成!");
            conf
        };
        ///初始化mysql数据库连接池
        static ref POOL : MySqlPool = {
            let ip = CONF_MAP.get_str("mysql_ip","");
            let user = CONF_MAP.get_str("mysql_user","");
            let pass = CONF_MAP.get_str("mysql_pass","");
            let port = CONF_MAP.get_usize("mysql_port",3306);
            let name = CONF_MAP.get_str("mysql_database_name","slg_http");
            let database_url = format!("mysql://{}:{}@{}:{}/{}", user, pass.as_str(), ip,port,name);
            let m = async{
               let res = sqlx::MySqlPool::connect(database_url.as_str()).await.unwrap();
               res
            };
            let pool = async_std::task::block_on(m);

            log::info!("初始化数据库完成!");
            pool
        };
}

fn main() {
    let time = std::time::Instant::now();
    //初始化日志
    init_log();
    //初始化数据库
    init_db();
    //初始化httpserver
    init_http_server();
    log::info!("httpserver启动成功!耗时:{}ms", time.elapsed().as_millis());
    std::thread::park();
}

fn init_db() {
    POOL.is_closed();
    db::check();
}

pub fn init_http_server() {
    http_server::init_server();
}

pub fn init_log() {
    let path = env::current_dir()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
        .to_owned()
        + "\\log_config.yaml";
    tools::my_log::init_log(path.as_str());
}
