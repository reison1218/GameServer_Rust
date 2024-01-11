use std::collections::HashMap;

use crate::entity::server_info::{self, ServerInfo};

use super::*;

pub struct AddServerHandler;

impl HttpServerHandler for AddServerHandler {
    fn get_path(&self) -> &str {
        "/slg/add_server"
    }

    fn do_get(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        log::info!("收到add_server,uri:{:?}", _uri);
        if _uri_params.is_empty() {
            return Ok(r#"{"status": "fail!","mess","缺少参数！"}"#.to_string());
        }
        let server_id_str = _uri_params.get("server_id");
        let name = _uri_params.get("name");
        let ws = _uri_params.get("ws");
        let open_time = _uri_params.get("open_time");
        let register_state = _uri_params.get("register_state");
        let r#type = _uri_params.get("type");
        let manager = _uri_params.get("manager");
        let inner_manager = _uri_params.get("inner_manager");
        let server_type_str = _uri_params.get("server_type");

        if server_id_str.is_none() {
            return Ok(r#"{"status": "fail!","mess":"缺少server_id参数!"}"#.to_string());
        }
        if server_id_str.unwrap().parse::<i32>().is_err() {
            return Ok(r#"{"status": "fail!","mess":"server_id不是数字!"}"#.to_string());
        }
        let server_id = server_id_str.unwrap().parse::<i32>().unwrap();

        if name.is_none() {
            return Ok(r#"{"status": "fail!","mess":"name是空的!"}"#.to_string());
        }

        if ws.is_none() {
            return Ok(r#"{"status": "fail!","mess":"ws是空的!"}"#.to_string());
        }

        if open_time.is_none() {
            return Ok(r#"{"status": "fail!","mess":"open_time是空的!"}"#.to_string());
        }
        let open_time = open_time.unwrap();
        let mut open_time = open_time.replace("%20", " ");
        open_time = open_time.replace("%2B", " ");
        let res = chrono::NaiveDateTime::parse_from_str(open_time.as_str(), "%Y-%m-%d %H:%M:%S");
        if let Err(_) = res {
            return Ok(r#"{"status": "fail!","mess":"open_time格式有问题,请按照格式来！%Y-%m-%d %H:%M:%S"}"#.to_string());
        }
        let open_time = res.unwrap();

        if register_state.is_none() {
            return Ok(r#"{"status": "fail!","mess":"register_state是空的!"}"#.to_string());
        }

        if r#type.is_none() {
            return Ok(r#"{"status": "fail!","mess":"type是空的!"}"#.to_string());
        }

        if manager.is_none() {
            return Ok(r#"{"status": "fail!","mess":"manager是空的!"}"#.to_string());
        }

        if inner_manager.is_none() {
            return Ok(r#"{"status": "fail!","mess":"inner_manager是空的!"}"#.to_string());
        }

        if server_type_str.is_none() {
            return Ok(r#"{"status": "fail!","mess":"server_type是空的!"}"#.to_string());
        }

        if server_type_str.unwrap().parse::<i32>().is_err() {
            return Ok(r#"{"status": "fail!","mess":"merge_times不是数字!"}"#.to_string());
        }
        let server_type = server_type_str.unwrap().parse::<i32>().unwrap();

        let server_info = server_info::query(server_id);
        let server_info = match server_info {
            Some(mut server_info) => {
                server_info.name = name.unwrap().to_string();
                server_info.ws = ws.unwrap().to_string();
                server_info.open_time = open_time;
                server_info.register_state = register_state.unwrap().parse::<i32>().unwrap();
                server_info.r#type = r#type.unwrap().to_string();
                server_info.manager = manager.unwrap().to_string();
                server_info.inner_manager = inner_manager.unwrap().to_string().replace("?", "");
                server_info.server_type = server_type;
                server_info
            }
            None => {
                let mut server_info = ServerInfo::default();
                server_info.server_id = server_id;
                server_info.name = name.unwrap().to_string();
                server_info.ws = ws.unwrap().to_string();
                server_info.open_time = open_time;
                server_info.register_state = register_state.unwrap().parse::<i32>().unwrap();
                server_info.state = 0;
                server_info.letter = 0;
                server_info.target_server_id = 0;
                server_info.r#type = r#type.unwrap().to_string();
                server_info.manager = manager.unwrap().to_string();
                server_info.inner_manager = inner_manager.unwrap().to_string().replace("?", "");
                server_info.server_type = server_type;
                server_info
            }
        };

        let m = async {
            server_info::insert(&server_info).await;
        };
        async_std::task::block_on(m);

        let url = crate::CONF_MAP.get_str("reload_http_list_url", "");
        let res = tools::http::send_get(url.as_str(), None, None);
        if let Err(err) = res {
            log::error!("{:?}", err);
        }
        Ok(r#"{"status":"success！"}"#.to_string())
    }
}
