use std::time::Duration;
use super::*;
use crate::PACKAGE_NAME;
use crate::check::pay::{validate_purchase_response, verify_google_purchase};
use log::info;
use serde_json::json;

pub struct CheckPayHandler;
impl HttpServerHandler for CheckPayHandler {
    fn get_path(&self) -> &str {
        "/gg_check/check_pay"
    }

    fn do_get(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        let product_id = _uri_params.get("product_id");
        if product_id.is_none() {
            let res = build_res(-100, "没有product_id!", None).to_string();
            return Ok(res);
        }
        let purchase_token = _uri_params.get("purchase_token");
        if purchase_token.is_none() {
            let res = build_res(-100, "没有purchase_token!", None).to_string();
            return Ok(res);
        }
        let product_id = product_id.unwrap();
        let purchase_token = purchase_token.unwrap();

        let read_lock = crate::SERVER_TOKEN.read().unwrap();

        let access_token = read_lock.access_token.clone();
        drop(read_lock);
        let is_subscription = false;

        // 3. 验证购买
        let purchase_response = verify_google_purchase(
            &access_token,
            PACKAGE_NAME.as_str(),
            product_id,
            purchase_token,
            is_subscription,
        );

        if let Err(e) = purchase_response {
            return Err(e);
        }
        let purchase_response = purchase_response.unwrap();

        // 验证响应数据
        let res = validate_purchase_response(&purchase_response, is_subscription);
        if let Err(e) = res {
            let err_str = format!("{:?}", e);
            error!("{:?}", err_str);
            let res = build_res(-100, err_str.as_str(), None).to_string();
            return Ok(res);
        }
        //给谷歌发送确认
        let url = format!("https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{}/purchases/products/{}/tokens/{}:acknowledge", PACKAGE_NAME.as_str(), product_id, purchase_token);
        let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(5)).build()?;
        // 3. 构建请求体 (可选developerPayload)
        let request_body = json!({
                "developerPayload": "your_custom_data_here" // 可选字段
            });
        let read = crate::SERVER_TOKEN.read().unwrap();
        let access_token = read.access_token.clone();
        drop(read);
        let response = client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&request_body)
            .send()
            ?;
        // 5. 处理响应
        if response.status().is_success() {
            info!("✅ 购买确认成功！");
        } else {
            let status = response.status();
            let error_text = response.text()?;
            error!("❌ 确认失败: {}   {}", status,error_text);
        }
        let res = build_res(200, "success", Some(purchase_response.to_json())).to_string();
        Ok(res)
    }
}