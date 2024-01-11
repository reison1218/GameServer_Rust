use tools::http::send_get;

use crate::entity::server_info;

use super::*;
pub struct GameYearlyHandler;

impl HttpServerHandler for GameYearlyHandler {
    fn get_path(&self) -> &str {
        "/slg/game_yearly"
    }

    fn do_get(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        if _uri_params.is_empty() {
            return Ok(r#"{"status": "fail!","mess":"缺少参数！"}"#.to_string());
        }

        let server_list_str = _uri_params.get("server_list");
        if server_list_str.is_none() {
            return Ok(r#"{"status": "fail!","mess":"缺少server_list参数!"}"#.to_string());
        }
        let server_list_str = server_list_str.unwrap();

        let read = server_info::querys(server_list_str.to_string());
        read.values()
            .filter(|x| x.target_server_id == 0)
            .for_each(|server_info| {
                let mut url = String::new();
                url.push_str(server_info.inner_manager.as_str());
                url.push_str("/api/merge/cal_yearly");
                let _ = send_get(url.as_str(), None, None);
            });
        Ok(r#"{"status":"success!"}"#.to_string())
    }
}
