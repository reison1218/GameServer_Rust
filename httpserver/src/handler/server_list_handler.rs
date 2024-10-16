use std::collections::HashMap;

use super::*;

use crate::entity::user;

///微信用户订阅handler
pub struct ServerListHandler;

impl HttpServerHandler for ServerListHandler {
    fn get_path(&self) -> &str {
        "/slg/server_list"
    }

    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<JsonValue> {
        let _json_params = JsonValue::from_bytes(_json_params).unwrap();
        let acc = _json_params.get_str("acc");
        let type_str = _json_params.get_str("type");
        let server_type = _json_params.get_i32("server_type");

        if acc.is_none() {
            return Ok(json!(
                r#"{"result": "fail!","errMessage","the acc is null!"}"#
            ));
        }
        if type_str.is_none() {
            return Ok(json!(
                r#"{"result": "fail!","errMessage","the type is null!"}"#
            ));
        }
        if server_type.is_none() {
            return Ok(json!(
                r#"{"result": "fail!","errMessage","the server_type is null!"}"#
            ));
        }

        let acc = acc.unwrap();
        let type_str = type_str.unwrap();
        let server_type = server_type.unwrap();

        let res = type_str.split(",");
        let mut server_list = Vec::new();

        res.into_iter().for_each(|s| server_list.push(s.to_owned()));
        let ctime = chrono::Local::now().timestamp_micros();

        //获取玩家userjson
        let mut user_login_map_write = crate::USER_LOGIN_MAP.write().unwrap();
        let res = user_login_map_write.get(acc);
        if res.is_none() {
            let res = user::find_user_login_info(acc);
            user_login_map_write.insert(acc.to_owned(), res);
        }
        let user_json = user_login_map_write.get(acc).unwrap();

        //临时变量
        let mut player_name = None;
        let mut last_login_time = 0u64;
        let mut level = None;
        let mut server_json_array = Vec::<JsonValue>::new();

        let server_map = crate::SERVER_MAP.read().unwrap();

        server_map
            .iter()
            .filter(|(_, server_info)| server_info.can_show(acc, &server_list, server_type, ctime))
            .for_each(|(_, server_info)| {
                let mut server_json = server_info.to_json();

                let server_id_str = server_info.server_id.to_string();

                let user_server_info_json = user_json.get_object(server_id_str.as_str());
                if let Some(user_server_info_json) = user_server_info_json {
                    player_name = user_server_info_json.get_str("player_name");
                    if let Some(v) = user_server_info_json.get_u64("login_time") {
                        last_login_time = v;
                    }
                    level = user_server_info_json.get_u16("level");
                }
                server_json.insert(
                    "last_login_time".to_owned(),
                    JsonValue::from(last_login_time),
                );
                if player_name.is_some() {
                    server_json.insert(
                        "player_name".to_owned(),
                        JsonValue::from(player_name.clone()),
                    );
                    server_json.insert("level".to_owned(), JsonValue::from(level.unwrap()));
                }
                server_json_array.push(server_json);
            });
        drop(user_login_map_write);

        let mut return_json = serde_json::Map::new();

        return_json.insert("data".to_owned(), JsonValue::from(server_json_array));
        return_json.insert("status".to_owned(), JsonValue::from("OK"));
        Ok(JsonValue::from(return_json))
    }
}
