use std::collections::HashMap;

use serde_json::json;
use tools::http::HttpServerHandler;

pub struct ReloadHandler;
impl HttpServerHandler for ReloadHandler {
    fn get_path(&self) -> &str {
        "/slg/reload"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<serde_json::Value> {
        Ok(json!(r#"{"statue","success"}"#))
    }

    fn do_get(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        crate::reload();
        Ok(r#"{"statue","success"}"#.to_string())
    }
}
