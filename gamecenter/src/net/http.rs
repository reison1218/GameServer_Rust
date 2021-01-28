use super::*;

use async_std::sync::Mutex;
use async_std::task::block_on;
use http_types::Error as HttpTypesError;
use serde_json::{json, Value};
use tools::http::HttpServerHandler;

pub struct StopAllServerHandler {
    gm: Arc<Mutex<GameCenterMgr>>,
}

impl StopAllServerHandler {
    pub fn new(gm: Arc<Mutex<GameCenterMgr>>) -> Self {
        StopAllServerHandler { gm }
    }
}


impl HttpServerHandler for StopAllServerHandler {
    fn get_path(&self) -> &str {
        "reload_temps"
    }

    fn execute(
        &mut self,
        _: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        let mut lock = block_on(self.gm.lock());
        lock.stop_all_server_handler();
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}

pub struct ReloadTempsHandler {
    gm: Arc<Mutex<GameCenterMgr>>,
}

impl ReloadTempsHandler {
    pub fn new(gm: Arc<Mutex<GameCenterMgr>>) -> Self {
        ReloadTempsHandler { gm }
    }
}


impl HttpServerHandler for ReloadTempsHandler {
    fn get_path(&self) -> &str {
        "reload_temps"
    }

    fn execute(
        &mut self,
        _: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        let mut lock = block_on(self.gm.lock());
        lock.notice_reload_temps();
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}

pub struct UpdateSeasonHandler {
    gm: Arc<Mutex<GameCenterMgr>>,
}

impl UpdateSeasonHandler {
    pub fn new(gm: Arc<Mutex<GameCenterMgr>>) -> Self {
        UpdateSeasonHandler { gm }
    }
}

impl HttpServerHandler for UpdateSeasonHandler {
    fn get_path(&self) -> &str {
        "update_season"
    }

    fn execute(
        &mut self,
        data: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        let mut lock = block_on(self.gm.lock());
        lock.notice_update_season(data.unwrap());
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}