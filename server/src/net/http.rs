use crate::entity::save_player_http;
use crate::{Lock, CONF_MAP};
use http_types::Error as HttpTypesError;
use log::{error, info};
use serde_json::value::Value as JsonValue;
use serde_json::{json, Map};
use std::time::Duration;
use tools::http::{HttpMethod, HttpServerHandler};

///通知用户中心类型
pub enum UserCenterNoticeType {
    Login,
    OffLine,
}

///保存玩家数据
pub struct SavePlayerHttpHandler {
    gm: Lock,
}

impl SavePlayerHttpHandler {
    pub fn new(gm: Lock) -> Self {
        SavePlayerHttpHandler { gm }
    }
}

impl HttpServerHandler for SavePlayerHttpHandler {
    fn get_path(&self) -> &str {
        "save"
    }

    fn execute(&mut self, _: Option<JsonValue>) -> Result<JsonValue, http_types::Error> {
        save_player_http(self.gm.clone());
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}

pub struct StopServerHttpHandler {
    gm: Lock,
}

impl StopServerHttpHandler {
    pub fn new(gm: Lock) -> Self {
        StopServerHttpHandler { gm }
    }
}

impl HttpServerHandler for StopServerHttpHandler {
    fn get_path(&self) -> &str {
        "exit"
    }

    fn execute(
        &mut self,
        _: Option<JsonValue>,
    ) -> core::result::Result<serde_json::Value, HttpTypesError> {
        save_player_http(self.gm.clone());
        let value = json!({ "status":"OK" });
        let exit = async {
            async_std::task::sleep(Duration::from_secs(3)).await;
            info!("游戏服务器退出进程!");
            std::process::exit(1);
        };
        async_std::task::spawn(exit);
        Ok(value)
    }
}

///异步通知用户中心
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
