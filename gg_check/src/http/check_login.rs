use super::*;
use crate::check::login::{exchange_code_for_token, verify_google_id_token};

pub struct CheckLoginHandler;
impl HttpServerHandler for CheckLoginHandler {
    fn get_path(&self) -> &str {
        "/gg_check/check_login"
    }

    fn do_get(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        let token = _uri_params.get("token");
        if token.is_none() {
            let res = build_res(-100, "没有token!", None).to_string();
            return Ok(res);
        }
        let token = token.unwrap();

        let res = exchange_code_for_token(token);
        if let Err(e) = res {
            error!("{:?}", e);
            anyhow::bail!("exchange_code_for_token error: {}", e);
        }
        let res = res.unwrap();
        if res.id_token.is_none() {
            let err_mess  = format!("id_token missing!! token:{}  res:{:?}",token,res);
            anyhow::bail!("exchange_code_for_token error: {}", err_mess);
        }
        let res = verify_google_id_token(res.id_token.as_ref().unwrap().as_str());

        if let Err(e) = &res {
            let err_str = format!("{:?}", e);
            error!("{:?}", err_str);
            let res = build_res(-100, err_str.as_str(), None).to_string();
            return Ok(res);
        }
        let res = res?;
        let res = build_res(200, "success", Some(res.to_json())).to_string();
        Ok(res)
    }
}