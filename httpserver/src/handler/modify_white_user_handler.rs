use std::collections::HashMap;

use crate::entity::white_user::{self};

use super::*;

pub struct ModifyWhiteUserHandler;

impl HttpServerHandler for ModifyWhiteUserHandler {
    fn get_path(&self) -> &str {
        "/slg/modify_white_user"
    }

    fn do_get(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        if _uri_params.is_empty() {
            return Ok(r#"{"status": "fail!","mess":"缺少参数！"}"#.to_string());
        }

        let name = _uri_params.get("name");
        let r#type = _uri_params.get("type");

        if name.is_none() {
            return Ok(r#"{"status": "fail!","mess":"缺少name参数！"}"#.to_string());
        }
        if r#type.is_none() {
            return Ok(r#"{"status": "fail!","mess":"缺少type参数！"}"#.to_string());
        }
        let name = name.unwrap();
        let r#type = r#type.unwrap();

        if r#type.eq("1") {
            white_user::insert(name);
        } else {
            white_user::delete(name);
        }
        let url = crate::CONF_MAP.get_str("reload_http_list_url", "");
        let res = tools::http::send_get(url.as_str(), None, None);
        if let Err(err) = res {
            log::error!("{:?}", err);
        }
        return Ok(r#"{"status":"success!"}"#.to_string());
    }
}
