use std::collections::HashMap;

use anyhow::Ok;
use tools::{http::HttpServerHandler, json::JsonValue};

use crate::entity::{server_info, user, wx_user_subscribe};

static ARENA_FIGHTTASK_ATTACKCITY: &str = "muyYTSpN35jEaRl_q3vNLMF9Job6vVyWF50fjvVY_RE";

static AUCTION: &str = "TNNJufhO4WlmMl8hVxJMgkLOaTswHcbijcP6BuQhVBY";

static KING_WAR: &str = "nTcGEUET4c03vnpVmLLqCtVzHvAi4fe3o4fRyEpLm00";

///通知消息handler,应该是从游戏服那边发送过来的
pub struct NoticeMessHandler;

impl HttpServerHandler for NoticeMessHandler {
    fn get_path(&self) -> &str {
        "/slg/wx_message"
    }

    fn do_get(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        log::info!("收到wx_message请求！");
        let key = _uri_params.get("key");
        let server_id = _uri_params.get("server_id");
        if key.is_none() {
            return Ok(r#"{"code":-1,"msg":"找不到key參數"}"#.to_string());
        }
        if server_id.is_none() {
            return Ok(r#"{"code":-1,"msg":"找不到server_id參數"}"#.to_string());
        }
        let key = key.unwrap();
        let server_id = server_id.unwrap().parse::<i32>().unwrap();
        let mut temp_id = "";
        let mut j1 = serde_json::Map::new();
        let mut j2 = serde_json::Map::new();
        let mut j3 = serde_json::Map::new();
        let mut data = serde_json::Map::new();

        match key.as_str() {
            "arena" | "fightTask" | "attackCity" => {
                temp_id = ARENA_FIGHTTASK_ATTACKCITY;

                match key.as_str() {
                    "arena" => {
                        j1.insert("value".to_string(), JsonValue::from("擂台战"));
                        j1.insert("value".to_string(), JsonValue::from("擂台币"));
                    }
                    "fightTask" => {
                        j1.insert("value".to_string(), JsonValue::from("国战"));
                        j2.insert("value".to_string(), JsonValue::from("战功"));
                    }
                    "attackCity" => {
                        j1.insert("value".to_string(), JsonValue::from("金币异种入侵"));
                        j2.insert("value".to_string(), JsonValue::from("金币"));
                    }
                    _ => {}
                }

                j3.insert("value".to_string(), JsonValue::from("20:00"));
                data.insert("thing1".to_string(), JsonValue::from(j1));
                data.insert("thing5".to_string(), JsonValue::from(j2));
                data.insert("time8".to_string(), JsonValue::from(j3));
            }
            "auction" => {
                temp_id = AUCTION;
                j1.insert("value".to_string(), JsonValue::from("神将拍卖"));
                j2.insert("value".to_string(), JsonValue::from("神将拍卖即将结束"));
                data.insert("thing1".to_string(), JsonValue::from(j1));
                data.insert("thing2".to_string(), JsonValue::from(j2));
            }
            "kingWar" => {
                temp_id = KING_WAR;
                j1.insert("value".to_string(), JsonValue::from("称王战开启"));
                j2.insert(
                    "value".to_string(),
                    JsonValue::from("称王之战已开启，请火速上线"),
                );
                data.insert("thing1".to_string(), JsonValue::from(j1));
                data.insert("thing2".to_string(), JsonValue::from(j2));
            }
            _ => {}
        }

        let mut merge_list = server_info::query_merged_server_ids(server_id);
        merge_list.push(server_id);
        let name_list = user::query_name_by_server_ids(merge_list);
        let mut name_str = String::new();
        for (index, name) in name_list.iter().enumerate() {
            name_str.push_str(name.as_str());
            if index != name_list.len() - 1 {
                name_str.push_str(",");
            }
        }
        let mut white_users = wx_user_subscribe::querys_by_names(name_str);
        for name in name_list {
            let name = name.as_str();
            let info = white_users.get_mut(name);
            if info.is_none() {
                continue;
            }
            let info = info.unwrap();
            if !info.templ_ids.contains_key(temp_id) {
                continue;
            }

            let times = info.templ_ids.get(temp_id).unwrap();
            if *times < 1 {
                continue;
            }

            let res =
                crate::wx::send_subscribe(&info.open_id, temp_id, JsonValue::from(data.clone()));
            if res {
                info.templ_ids.insert(temp_id.to_string(), times - 1);
                log::info!(
                    "触发微信推送 key:{},接受到的账号：{}",
                    temp_id,
                    info.open_id.as_str()
                );
            }
        }
        Ok(r#"{"statue":"success"}"#.to_string())
    }
}
