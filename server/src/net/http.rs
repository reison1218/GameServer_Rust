use super::*;
use async_h1::client;
use http_types::{Body, Error as HttpTypesError, Method, Request, Response, StatusCode, Url};
use serde_json::json;
use serde_json::Value;
use std::time::Duration;
use tools::http::HttpServerHandler;
use crate::entity::save_player_http;
use crate::entity::Entity;

pub struct SavePlayerHttpHandler {
    gm: Arc<RwLock<GameMgr>>,
}

impl SavePlayerHttpHandler {
    pub fn new(gm: Arc<RwLock<GameMgr>>) -> Self {
        SavePlayerHttpHandler { gm }
    }
}

impl HttpServerHandler for SavePlayerHttpHandler {
    fn get_path(&self) -> &str {
        "save"
    }

    fn execute(
        &mut self,
        params: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        save_player_http(self.gm.clone());
        let mut value = json!({ "status":"OK" });
        Ok(value)
    }
}

pub struct StopPlayerHttpHandler {
    gm: Arc<RwLock<GameMgr>>,
}

impl StopPlayerHttpHandler {
    pub fn new(gm: Arc<RwLock<GameMgr>>) -> Self {
        StopPlayerHttpHandler { gm }
    }
}

impl  HttpServerHandler for StopPlayerHttpHandler {
    fn get_path(&self) -> &str {
        "exit"
    }

    fn execute(
        &mut self,
        params: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        save_player_http(self.gm.clone());
        let mut value = json!({ "status":"OK" });
        let exit = async {
            async_std::task::sleep(Duration::from_secs(3)).await;
            info!("游戏服务器退出进程!");
            std::process::exit(1);
        };
        async_std::task::spawn(exit);
        Ok(value)
    }
}
