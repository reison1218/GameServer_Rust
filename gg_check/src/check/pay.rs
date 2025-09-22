use anyhow::{Context, Result, anyhow};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use tools::json::{JsonValue, JsonValueTrait};
use crate::check::exchange_token;

// Google Play 验证响应结构
#[derive(Debug, Deserialize)]
pub struct GooglePurchaseResponse {
    #[serde(rename = "purchaseState")]
    pub purchase_state: Option<i32>,//0：已购买（Purchased）  1：已取消（Canceled）  2：待处理（Pending）
    #[serde(rename = "consumptionState")]
    pub consumption_state: Option<i32>,//0：未消耗（Yet to be consumed）  1：已消耗（Consumed）
    #[serde(rename = "orderId")]
    pub order_id: Option<String>,
    #[serde(rename = "developerPayload")]
    pub developer_payload: Option<String>,
    #[serde(rename = "acknowledgementState")]
    pub acknowledgement_state: Option<i32>,//0：未确认（Yet to be acknowledged） 1：已确认（Acknowledged）
    #[serde(rename = "kind")]
    pub kind: Option<String>,
}

impl GooglePurchaseResponse {
    pub fn to_json(&self) -> JsonValue {
        let mut map = JsonValue::new();
        if let Some(purchase_state) = self.purchase_state {
            map.insert("purchase_state".to_string(), Value::from(purchase_state));
        }
        if let Some(consumption_state) = self.consumption_state {
            map.insert(
                "consumption_state".to_string(),
                Value::from(consumption_state),
            );
        }
        if let Some(order_id) = self.order_id.clone() {
            map.insert("order_id".to_string(), Value::from(order_id));
        }
        if let Some(developer_payload) = self.developer_payload.clone() {
            map.insert(
                "developer_payload".to_string(),
                Value::from(developer_payload),
            );
        }
        if let Some(acknowledgement_state) = self.acknowledgement_state {
            map.insert(
                "acknowledgement_state".to_string(),
                Value::from(acknowledgement_state),
            );
        }
        if let Some(kind) = self.kind.clone() {
            map.insert("kind".to_string(), Value::from(kind));
        }
        map
    }
}


/// 验证 Google Play 购买
pub fn verify_google_purchase(
    access_token: &str,
    package_name: &str,
    product_id: &str,
    purchase_token: &str,
    is_subscription: bool,
) -> Result<GooglePurchaseResponse> {
    let endpoint = if is_subscription {
        format!(
            "https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{}/purchases/subscriptions/{}/tokens/{}",
            package_name, product_id, purchase_token
        )
    } else {
        format!(
            "https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{}/purchases/products/{}/tokens/{}",
            percent_encoding::utf8_percent_encode(package_name, percent_encoding::NON_ALPHANUMERIC),
            percent_encoding::utf8_percent_encode(product_id, percent_encoding::NON_ALPHANUMERIC),
            purchase_token
        )
    };
    let client = reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(5)).build()?;
    let response = client
        .get(&endpoint)
        .bearer_auth(&access_token)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send()?;

    if !response.status().is_success() {
        let error_text = response.text()?;
        return Err(anyhow!("Verification failed: {}", error_text));
    }

    let purchase_response: GooglePurchaseResponse = response.json()?;
    Ok(purchase_response)
}

/// 验证购买结果
pub fn validate_purchase_response(
    response: &GooglePurchaseResponse,
    is_subscription: bool,
) -> Result<()> {

    // 检查订单ID是否存在
    if response.order_id.is_none() {
        return anyhow::bail!("Missing order ID in response");
    }
    let order_id = response.order_id.as_ref().unwrap();

    if is_subscription {
        // 订阅验证逻辑
        // 有效状态: 1=已支付, 2=免费试用期
        if let Some(payment_state) = response.purchase_state {
            if payment_state != 1 && payment_state != 2 {
                anyhow::bail!("order_id: {} Invalid subscription state: {}", order_id,payment_state);
            }
        } else {
            anyhow::bail!("order_id: {} Missing purchase state in response", order_id);
        }

        // 检查确认状态 (0=未确认, 1=已确认)
        if response.acknowledgement_state != Some(1) {
            anyhow::bail!("order_id:{}Subscription not acknowledged", order_id);
        }
    } else {
        // 普通商品验证逻辑
        // 有效状态: 0=已购买
        if response.purchase_state != Some(0) {
            anyhow::bail!("order_id:{} Invalid purchase state:{}",order_id,response.purchase_state.as_ref().unwrap());
        }

        // 消耗状态: 0=未消耗 (需要发货)
        if response.consumption_state != Some(0) {
            anyhow::bail!("order_id:{} Product already consumed",order_id);
        }
    }
    Ok(())
}



/// 完整购买验证流程
pub fn verify_purchase(
    service_account_path: Option<&str>,
    package_name: &str,
    product_id: &str,
    purchase_token: &str,
    is_subscription: bool,
) -> Result<()> {
    log::info!("Starting Google Play purchase verification");

    let access_token;
    if service_account_path.is_some(){
        access_token = exchange_token(service_account_path.unwrap())?.access_token;
    }else{
        let read_lock = crate::SERVER_TOKEN.read().unwrap();
        access_token = read_lock.access_token.clone();
    }

    // 3. 验证购买
    let purchase_response = verify_google_purchase(
        &access_token,
        package_name,
        product_id,
        purchase_token,
        is_subscription,
    ).context("Failed to verify purchase")?;

    // 4. 验证响应数据
    validate_purchase_response(&purchase_response, is_subscription)
        .context("Purchase validation failed")?;

    // 5. 记录订单ID (防重放攻击)
    if let Some(order_id) = purchase_response.order_id {
        log::info!("Valid purchase with order ID: {}", order_id);
        // 在实际应用中，这里应该记录订单ID到数据库
    }

    log::info!("Purchase verified successfully");
    Ok(())
}
