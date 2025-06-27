use super::*;
use std::collections::HashMap;
use std::str::FromStr;

pub struct GatewayHandler;

impl HttpServerHandler for GatewayHandler {
    fn get_path(&self) -> &str {
        "/gateway/"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<serde_json::Value> {
        let json_params = JsonValue::from_bytes(_json_params);
        let relay_ip = crate::CONF_MAP.get_str("http_ip", "localhost");
        let url = format!("http://{}:8500{}", relay_ip, _uri);
        let res = match json_params {
            Ok(json_params) => {
                let res = tools::http::send_post(url.as_str(), Some(json_params)).unwrap();
                res
            }
            Err(_) => {
                let res = tools::http::send_post(url.as_str(), None).unwrap();
                res
            }
        };
        let json_res = tools::json::JsonValue::from_str(res.as_str()).unwrap();
        Ok(json_res)
    }
}
