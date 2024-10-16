pub mod db;
pub mod entity;
pub mod handler;
pub mod http_server;
pub mod wx;
use std::{
    collections::HashMap,
    env::{self},
    sync::{Arc, RwLock},
};

use crate::entity::{
    server_info::{self, ServerInfo},
    white_user::{self, WhiteUserInfo},
    wx_user_subscribe::{self, WxUsersSubscribe},
};
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

        ///初始化mgr
        static ref USER_LOGIN_MAP : Arc<RwLock<HashMap<String, serde_json::Value>>> = {
            Arc::new(RwLock::new(HashMap::new()))
        };
        ///初始化mgr
        static ref WX_USERS_SUBSCRIBE_MAP : Arc<RwLock<HashMap<String, WxUsersSubscribe>>> = {
            Arc::new(RwLock::new(HashMap::new()))
        };
        ///初始化mgr
        static ref SERVER_MAP : Arc<RwLock<HashMap<i32, ServerInfo>>> = {
            Arc::new(RwLock::new(HashMap::new()))
        };
        ///初始化mgr
        static ref WHITE_USER_MAP : Arc<RwLock<HashMap<String, WhiteUserInfo>>> = {
             Arc::new(RwLock::new(HashMap::new()))
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

pub fn reload() {
    let time = std::time::Instant::now();
    //reload server_list
    let server_infos = server_info::query_all();
    let mut write = crate::SERVER_MAP.write().unwrap();
    write.clear();
    for (server_id, server_info) in server_infos {
        write.insert(server_id, server_info);
    }
    drop(write);

    //reload 白名单
    let res = white_user::query_all();
    let mut write = crate::WHITE_USER_MAP.write().unwrap();
    write.clear();
    for info in res {
        write.insert(info.name.clone(), info);
    }
    drop(write);

    //reload 微信订阅
    let res = wx_user_subscribe::query();
    let mut write = crate::WX_USERS_SUBSCRIBE_MAP.write().unwrap();
    write.clear();
    for wx in res {
        write.insert(wx.name.clone(), wx);
    }
    drop(write);

    //reload users
    let mut write = crate::USER_LOGIN_MAP.write().unwrap();
    write.clear();
    log::info!("reload数据完成！耗时:{}ms", time.elapsed().as_millis());
}
