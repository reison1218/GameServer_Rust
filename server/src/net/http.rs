use crate::entity::save_player_http;
use crate::Lock;
use http_types::Error as HttpTypesError;
use log::info;
use serde_json::json;
use serde_json::value::Value as JsonValue;
use std::time::Duration;
use tools::http::HttpServerHandler;

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
