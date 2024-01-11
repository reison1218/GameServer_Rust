use crate::entity::server_info::{self};

use super::*;

pub struct ModifyServerHandler;

impl HttpServerHandler for ModifyServerHandler {
    fn get_path(&self) -> &str {
        "/slg/modify_server"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<JsonValue> {
        log::info!("收到modify_server,uri:{:?}", _uri);
        let _json_params = JsonValue::from_bytes(_json_params);
        if let Err(err) = _json_params {
            log::warn!("{:?}", err);
            return Ok(json!(r#"{"result": "fail!","errMessage":"参数有问题!"}"#));
        }
        let _json_params = _json_params.unwrap();

        let params_array = _json_params.as_array();
        if params_array.is_none() {
            return Ok(json!(r#"{"status": "fail!":"mess":"json参数不是数组!"}"#));
        }
        let params_array = params_array.unwrap();

        let mut return_json = JsonValue::default();

        let mut err_json = None;
        params_array.iter().for_each(|json| {
            let open_time = json.get_str("open_time");

            if let Some(open_time) = open_time {
                let mut open_time = open_time.replace("%20", " ");
                open_time = open_time.replace("%2B", " ");
                let res =
                    chrono::NaiveDateTime::parse_from_str(open_time.as_str(), "%Y-%m-%d %H:%M:%S");
                if let Err(_) = res {
                    let mut map = serde_json::Map::new();
                    map.insert("result".to_string(), JsonValue::from("fail!"));
                    map.insert(
                        "mess".to_string(),
                        JsonValue::from(
                            "open_time不是时间类型?请按格式来：%Y-%m-%d %H:%M:%S".to_string(),
                        ),
                    );
                    err_json = Some(JsonValue::from(map));
                    return;
                }
            }
        });
        if let Some(err_json) = err_json {
            return Ok(err_json);
        }

        for json in params_array.iter() {
            let server_id = json.get_i32("server_id");
            let name = json.get_str("name");
            let ws = json.get_str("ws");
            let open_time = json.get_str("open_time");
            let register_state = json.get_i32("register_state");
            let state = json.get_i32("state");
            let letter = json.get_i32("letter");
            let target_server_id = json.get_i32("target_server_id");
            let _merge_times = json.get_i32("merge_times");
            let r#type = json.get_str("type");
            let manager = json.get_str("manager");
            let inner_manager = json.get_str("inner_manager");
            let server_type = json.get_i32("server_type");

            let server_id = match server_id {
                Some(server_id) => server_id,
                None => 0,
            };

            let server_info = server_info::query(server_id);
            if server_info.is_none() {
                let mut map = serde_json::Map::new();
                map.insert("result".to_string(), JsonValue::from("fail!"));
                map.insert(
                    "mess".to_string(),
                    JsonValue::from("找不到对应server_id的数据!".to_string()),
                );
                return Ok(JsonValue::from(map));
            }

            let mut server_info = server_info.unwrap();
            if let Some(open_time) = open_time {
                let mut open_time = open_time.replace("%20", " ");
                open_time = open_time.replace("%2B", " ");
                server_info.open_time =
                    chrono::NaiveDateTime::parse_from_str(open_time.as_str(), "%Y-%m-%d %H:%M:%S")
                        .unwrap();
            }

            if let Some(name) = name {
                server_info.name = name.to_owned();
            }

            if let Some(ws) = ws {
                server_info.ws = ws.to_owned();
            }
            if let Some(ws) = ws {
                server_info.ws = ws.to_owned();
            }

            if let Some(register_state) = register_state {
                server_info.register_state = register_state;
            }
            if let Some(state) = state {
                server_info.state = state;
            }
            if let Some(letter) = letter {
                server_info.letter = letter;
            }
            if let Some(target_server_id) = target_server_id {
                server_info.target_server_id = target_server_id;
            }
            if let Some(server_type) = server_type {
                server_info.server_type = server_type;
            }
            if let Some(value) = r#type {
                server_info.r#type = value.to_owned();
            }
            if let Some(manager) = manager {
                server_info.manager = manager.to_owned();
            }
            if let Some(inner_manager) = inner_manager {
                log::info!("modify里面的inner_manager：{}", inner_manager);
                server_info.inner_manager = inner_manager.to_owned().replace("?", "");
            }
            let m = async {
                server_info::insert(&server_info).await;
            };
            async_std::task::block_on(m);
            return_json = json!(r#"{"status":"success!"}"#);
        }

        let url = crate::CONF_MAP.get_str("reload_http_list_url", "");
        let res = tools::http::send_get(url.as_str(), None, None);
        if let Err(err) = res {
            log::error!("{:?}", err);
        }
        Ok(return_json)
    }
}
