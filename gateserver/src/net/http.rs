use super::*;

use async_std::sync::RwLock;
use async_std::task::block_on;
use http_types::Error as HttpTypesError;
use serde_json::value::Value as JsonValue;
use serde_json::Value;
use serde_json::{json, Map};
use tools::http::HttpServerHandler;

pub struct KickPlayerHttpHandler {
    gm: Arc<RwLock<ChannelMgr>>,
}

impl KickPlayerHttpHandler {
    pub fn new(gm: Arc<RwLock<ChannelMgr>>) -> Self {
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
        let mut lock = block_on(self.gm.write());
        lock.kick_all();
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}

pub struct ReloadTempsHandler {
    gm: Arc<RwLock<ChannelMgr>>,
}

impl ReloadTempsHandler {
    pub fn new(gm: Arc<RwLock<ChannelMgr>>) -> Self {
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
        let mut lock = block_on(self.gm.write());
        lock.notice_reload_temps();
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}

pub struct UpdateSeasonHandler {
    gm: Arc<RwLock<ChannelMgr>>,
}

impl UpdateSeasonHandler {
    pub fn new(gm: Arc<RwLock<ChannelMgr>>) -> Self {
        UpdateSeasonHandler { gm }
    }
}

impl HttpServerHandler for UpdateSeasonHandler {
    fn get_path(&self) -> &str {
        "reload_temps"
    }

    fn execute(
        &mut self,
        data: Option<Value>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        let mut lock = block_on(self.gm.write());
        lock.notice_update_season(data.unwrap());
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}

///异步通知用户中心
pub async fn notice_user_center(user_id: u32, _type: &str) {
    let mut login = false;
    if _type.eq("login") {
        login = true;
    }
    modify_redis_user(user_id, login);
    //通知用户中心
    let http_port: &str = CONF_MAP.get_str("user_center_state");
    let game_id: usize = CONF_MAP.get_usize("game_id");
    let mut map: Map<String, JsonValue> = Map::new();
    map.insert("user_id".to_owned(), JsonValue::from(user_id));
    map.insert("game_id".to_owned(), JsonValue::from(game_id));
    map.insert("type".to_owned(), JsonValue::from(_type));
    let value = JsonValue::from(map);
    let res =
        tools::http::send_http_request(http_port, "center/user_state", "post", Some(value)).await;
    match res {
        Err(e) => {
            error!("{:?}", e.to_string());
        }
        Ok(_) => {}
    }
}
