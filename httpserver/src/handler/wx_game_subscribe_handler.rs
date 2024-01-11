use std::collections::HashMap;

use anyhow::Ok;
use serde_json::json;
use tools::{
    http::HttpServerHandler,
    json::{JsonValue, JsonValueTrait},
};

use crate::entity::wx_user_subscribe::{self, WxUsersSubscribe};

///微信用户订阅handler
pub struct WxGameSubscribeHandler;

impl HttpServerHandler for WxGameSubscribeHandler {
    fn get_path(&self) -> &str {
        "/slg/wx_subscribe"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<serde_json::Value> {
        let _json_params = JsonValue::from_bytes(_json_params);
        if let Err(err) = _json_params {
            log::warn!("{:?}", err);
            return Ok(json!(r#"{"result": "fail!","errMessage":"参数有问题!"}"#));
        }
        let _json_params = _json_params.unwrap();

        let account = _json_params.get_str("account");
        let open_id = _json_params.get_str("open_id");
        let templ_ids = _json_params.get_array("templIds");

        let mut return_json = serde_json::Map::new();
        if account.is_none() {
            log::warn!("微信订阅,account是空的?");
            return_json.insert("result".to_string(), JsonValue::from("fail!"));
            return_json.insert(
                "errMessage".to_string(),
                JsonValue::from("微信订阅,account是空的?!"),
            );
            return Ok(JsonValue::from(return_json));
        }

        if open_id.is_none() {
            log::warn!("微信订阅,open_id是空的?");
            return_json.insert("result".to_string(), JsonValue::from("fail!"));
            return_json.insert(
                "errMessage".to_string(),
                JsonValue::from("微信订阅,open_id是空的?!"),
            );
            return Ok(JsonValue::from(return_json));
        }

        if templ_ids.is_none() || templ_ids.unwrap().is_empty() {
            log::warn!("微信订阅,templ_ids是空的?");
            return_json.insert("result".to_string(), JsonValue::from("fail!"));
            return_json.insert(
                "errMessage".to_string(),
                JsonValue::from("微信订阅,templ_ids是空的?!".to_string()),
            );
            return Ok(JsonValue::from(return_json));
        }
        let account = account.unwrap();
        let open_id = open_id.unwrap();
        let templ_ids = templ_ids.unwrap();

        let mut wx = WxUsersSubscribe::default();
        wx.open_id = open_id.to_string();
        wx.name = account.to_string();

        templ_ids.iter().for_each(|value| {
            wx.add_templ_id(value.as_str().unwrap());
        });

        let m = async {
            wx_user_subscribe::insert(&wx).await;
        };
        async_std::task::block_on(m);

        Ok(json!(r#"{"statue":"success"}"#))
    }
}
