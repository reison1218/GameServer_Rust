pub mod check_login;
pub mod check_pay;
mod test;

use log::{error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use tools::http::HttpServerHandler;
use tools::json::{JsonValue, JsonValueTrait};
use crate::http::check_login::CheckLoginHandler;
use crate::http::check_pay::CheckPayHandler;
use crate::http::test::TestHandler;

fn build_res(code: i32, mess: &str, data: Option<JsonValue>) -> JsonValue {
    let mut res = JsonValue::new();
    res.insert("code".to_string(), JsonValue::from(code));
    res.insert("mess".to_string(), JsonValue::from(mess.to_string()));
    if data.is_some() {
        res.insert("data".to_string(), data.unwrap());
    }
    res
}

pub fn init_server() {
    let port = crate::CONF_MAP.get_usize("http_listen_port", 16888);
    tools::http::Builder::new()
        .route(Box::new(CheckLoginHandler))
        .route(Box::new(CheckPayHandler))
        .route(Box::new(TestHandler))
        .bind(port as u16);
}

