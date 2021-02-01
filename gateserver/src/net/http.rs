use super::*;

use async_std::sync::Mutex;
use async_std::task::block_on;
use http_types::Error as HttpTypesError;
use serde_json::json;
use serde_json::Value;
use tools::http::HttpServerHandler;

pub struct KickPlayerHttpHandler {
    gm: Arc<Mutex<ChannelMgr>>,
}

impl KickPlayerHttpHandler {
    pub fn new(gm: Arc<Mutex<ChannelMgr>>) -> Self {
        KickPlayerHttpHandler { gm }
    }
}

impl HttpServerHandler for KickPlayerHttpHandler {
    fn get_path(&self) -> &str {
        "kick"
    }

    fn execute(
        &mut self,
        _: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        let mut lock = block_on(self.gm.lock());
        lock.kick_all();
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}
