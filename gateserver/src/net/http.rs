use super::*;

use async_std::sync::Mutex;
use async_std::task::block_on;
use http_types::Error as HttpTypesError;
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
use tools::http::HttpMethod;
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
    let http_port: &str = CONF_MAP.get_str("user_center_state");
    let game_id: usize = CONF_MAP.get_usize("game_id");
    let mut map: Map<String, JsonValue> = Map::new();
    map.insert("user_id".to_owned(), JsonValue::from(user_id));
    map.insert("game_id".to_owned(), JsonValue::from(game_id));
    map.insert("type".to_owned(), JsonValue::from(is_login));
    let value = JsonValue::from(map);
    let res = tools::http::send_http_request(
        "http://",
        http_port,
        "center/user_state",
        HttpMethod::POST,
        Some(value),
    )
    .await;
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
    fn get_method(&self) -> tools::http::HttpMethod {
        tools::http::HttpMethod::POST
    }

    fn do_post(&mut self, _: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let mut lock = block_on(self.gm.lock());
        lock.kick_all();
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}
