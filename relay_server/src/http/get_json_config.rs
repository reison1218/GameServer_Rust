use super::*;
use log::error;
use std::collections::HashMap;

pub struct GetJsonConfigHandler;

impl HttpServerHandler for GetJsonConfigHandler {
    fn get_path(&self) -> &str {
        "/get_json_config/:file_name"
    }

    fn do_get(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        let relay_ip = crate::CONF_MAP.get_str("http_ip", "localhost");
        let url = format!("http://{}:8500{}", relay_ip, _uri);
        let res = tools::http::send_get(url.as_str(), None, None);
        match res {
            Ok(res) => return Ok(res),
            Err(e) => {
                error!("{}", e);
            }
        };
        Ok(r#"{"statue":200,"mess":"success"}"#.to_string())
    }
}
