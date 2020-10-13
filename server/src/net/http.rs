use super::*;
use crate::entity::save_player_http;
use crate::helper::redis_helper::modify_redis_user;
use crate::CONF_MAP;
use http_types::Error as HttpTypesError;
use serde_json::value::Value as JsonValue;
use serde_json::Value;
use serde_json::{json, Map};
use std::sync::Mutex;
use std::time::Duration;
use tools::http::HttpServerHandler;

pub struct SavePlayerHttpHandler {
    gm: Arc<Mutex<GameMgr>>,
}

impl SavePlayerHttpHandler {
    pub fn new(gm: Arc<Mutex<GameMgr>>) -> Self {
        SavePlayerHttpHandler { gm }
    }
}

impl HttpServerHandler for SavePlayerHttpHandler {
    fn get_path(&self) -> &str {
        "save"
    }

    fn execute(&mut self, _: Option<Value>) -> Result<Value, http_types::Error> {
        save_player_http(self.gm.clone());
        let value = json!({ "status":"OK" });
        Ok(value)
    }
}

pub struct StopPlayerHttpHandler {
    gm: Arc<Mutex<GameMgr>>,
}

impl StopPlayerHttpHandler {
    pub fn new(gm: Arc<Mutex<GameMgr>>) -> Self {
        StopPlayerHttpHandler { gm }
    }
}

impl HttpServerHandler for StopPlayerHttpHandler {
    fn get_path(&self) -> &str {
        "exit"
    }

    fn execute(
        &mut self,
        _: Option<Value>,
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
pub async fn notice_user_center(user_id: u32, _type: &str) {
    let mut login = false;
    if _type.eq("login") {
        login = true;
    }
    modify_redis_user(user_id, "on_line".to_string(), Value::from(login));
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
