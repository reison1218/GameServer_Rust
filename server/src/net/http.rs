use crate::entity::save_player_http;
use crate::Lock;
use http_types::Error as HttpTypesError;
use log::info;
use serde_json::json;
use serde_json::value::Value as JsonValue;
use std::collections::HashMap;
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
        "/save"
    }

    fn do_get(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        save_player_http(self.gm.clone());
        let value = json!({ "status":"OK" });
        Ok(value.to_string())
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
        "/exit"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<serde_json::Value> {
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
