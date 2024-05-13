use std::collections::HashMap;

use super::*;

use async_std::sync::Mutex;
use async_std::task::block_on;
use serde_json::json;
use serde_json::Map;
use tools::http::HttpServerHandler;

type JsonValue = serde_json::Value;

///通知用户中心类型
pub enum UserCenterNoticeType {
    Login,
    OffLine,
}

pub async fn notice_user_center(user_id: u32, notice_type: UserCenterNoticeType) {
    let is_login;
    match notice_type {
        UserCenterNoticeType::Login => is_login = true,
        UserCenterNoticeType::OffLine => is_login = false,
    }
    //通知用户中心
    let http_port: &str = &CONF_MAP.get_str("user_center_state", "");
    let game_id: usize = CONF_MAP.get_usize("game_id", 0);
    let mut map: Map<String, JsonValue> = Map::new();
    map.insert("user_id".to_owned(), JsonValue::from(user_id));
    map.insert("game_id".to_owned(), JsonValue::from(game_id));
    map.insert("type".to_owned(), JsonValue::from(is_login));
    let value = JsonValue::from(map);

    let url = format!("http://{}{}", http_port, "center/user_state");

    let res = tools::http::send_post(url.as_str(), Some(value));
    match res {
        Err(e) => {
            error!("{:?}", e.to_string());
        }
        Ok(_) => {}
    }
}

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
        "/kick"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<serde_json::Value> {
        let mut lock = block_on(self.gm.lock());
        lock.kick_all();
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}
