use std::str::FromStr;

use tools::json::{JsonValue, JsonValueTrait};

use tools::http::{send_get, send_post};

static SEND_SUBSCRIBE_URL: &str = "https://api.weixin.qq.com/cgi-bin/message/subscribe/send";

pub fn get_access_token() -> Option<String> {
    let app_id = crate::CONF_MAP.get_str("app_id", "");
    let app_secret = crate::CONF_MAP.get_str("app_secret", "");
    let url = format!(
        "https://mp-weixin.q1.com/WXAPIService.asmx/GetAccessTokenS?appId={}&secret={}",
        app_id, app_secret
    );
    let res = send_get(&url, None, None);
    if let Err(err) = res {
        log::warn!("{:?}", err);
        return None;
    }
    let res = res.unwrap();
    if res.is_empty() {
        log::warn!("获取access_token！游戏圈那边返回的结果是空字符串！");
        return None;
    }

    Some(res)
}

pub fn send_subscribe(open_id: &str, temp_id: &str, data: serde_json::Value) -> bool {
    let access_token = get_access_token();
    if access_token.is_none() {
        log::warn!("无法获取微信access_token！");
        return false;
    }
    let access_token = access_token.unwrap();
    let url = format!("{}?access_token={}", SEND_SUBSCRIBE_URL, access_token);
    let mut param_map = serde_json::Map::new();
    param_map.insert("touser".to_string(), JsonValue::from(open_id));
    param_map.insert("template_id".to_string(), JsonValue::from(temp_id));
    param_map.insert("data".to_string(), data);
    param_map.insert("miniprogram_state".to_string(), JsonValue::from("formal"));

    let res = send_post(url.as_str(), Some(JsonValue::from(param_map)));
    if let Err(err) = res {
        log::error!("发送订阅失败！openId:{} 错误信息:{:?}", "open_id", err);
        return false;
    }
    let res = res.unwrap();
    if res.is_empty() {
        log::warn!(
            "发送订阅异常！游戏圈那边返回empty字符串！open_id:{}",
            "openId"
        );
        return false;
    }
    let res = serde_json::Value::from_str(&res);
    if let Err(_) = res {
        log::warn!("发送订阅异常!游戏圈那边返回的不是json数据类型?{:?}", res);
        return false;
    }
    let res = res.unwrap();
    let err_code = res.get_i32("errcode");
    let err_msg = res.get_str("errmsg");

    match err_code {
        Some(err_code) => {
            if err_code != 0 {
                log::warn!(
                    "发送订阅失败!open_id:{} 错误码:{} 错误信息:{}",
                    "openId",
                    err_code,
                    err_msg.unwrap()
                );
                return false;
            }
        }
        None => {
            log::info!("发送订阅成功!open_id:{}  mess:{:?}", "open_id", err_msg);
        }
    }
    true
}
