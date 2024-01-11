use std::collections::HashMap;

use tools::http::send_get;

use crate::entity::server_info;

use super::*;

pub struct QuestionnaireHandler;

impl HttpServerHandler for QuestionnaireHandler {
    fn get_path(&self) -> &str {
        "/slg/q1/questionnaire"
    }

    fn do_get(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        if _uri_params.is_empty() {
            return Ok(r#"{"status": "fail!","mess":"缺少参数！"}"#.to_string());
        }

        let server_id_str = _uri_params.get("server_id");
        if server_id_str.is_none() {
            return Ok(r#"{"status": "fail!","mess":"缺少server_id参数！"}"#.to_string());
        }

        let server_id;
        let res = server_id_str.unwrap().parse::<i32>();
        match res {
            Ok(id) => server_id = id,
            Err(_) => {
                return Ok(r#"{"status": "fail!","mess":"server_id不是数字！"}"#.to_string());
            }
        }

        let server_info = server_info::query_merge(server_id);
        if server_info.is_none() {
            return Ok(r#"{"code": "-1!","mess":"找不到相关server_id的记录！"}"#.to_string());
        }

        let server_info = server_info.unwrap();
        let mut quest_url = String::new();
        quest_url.push_str(server_info.inner_manager.as_str());
        quest_url.push_str("/api/common/questionnaire");
        let res = send_get(quest_url.as_str(), Some(_uri_params), None);
        if let Err(err) = res {
            log::warn!("调查问卷出错了！{:?}", err);
            return Ok(r#"{"status":"fail!"}"#.to_string());
        }
        Ok(res.unwrap())
    }
}
