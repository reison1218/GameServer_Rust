use log::log;
use reqwest::Client;
use serde_json::json;
use crate::check::exchange_token;
use crate::check::pay::verify_purchase;

#[test]
fn test_pay() {
    // 配置参数
    let service_account_path =
        "C:\\Users\\Administrator\\Desktop\\packet\\zqwz-467410-406042b26096.json";
    let package_name = "org.zqwz.cc";
    let product_id = "pay521";
    let purchase_token = "mbknjbiclciemjklnbiclgba.AO-J1Ozpyz7_3c7FT5QpSxQUqNm-HCEa3xl32-gz3YDNekNOky8zfAExMPXCRRfn_RPf2qCjHT7PmFDxnRIladqs202D4YrQ8A";
    let is_subscription = false; // 如果是订阅则设为 true


    let google_api_token = exchange_token(service_account_path).unwrap();

    // 验证购买
    let res = verify_purchase(
        Some(service_account_path), 
        package_name,
        product_id,
        purchase_token,
        is_subscription,
    );


    match res {
        Ok(_) => {
            println!("✅ Payment verified. Delivering product...");
            // 发货逻辑...并向谷歌发送acknowledgement_state为0确认订单，不然3天之后系统自动退款
            let url = format!("https://androidpublisher.googleapis.com/androidpublisher/v3/applications/{}/purchases/products/{}/tokens/{}:acknowledge",package_name,product_id,purchase_token);
            let client = Client::new();
            // 3. 构建请求体 (可选developerPayload)
            let request_body = json!({
                "developerPayload": "your_custom_data_here" // 可选字段
            });
            let client = reqwest::blocking::Client::new();
            let response = client
                .post(&url)
                .bearer_auth(&google_api_token.access_token)
                .json(&request_body)
                .send()
                .unwrap();

            // 5. 处理响应
            if response.status().is_success() {
                println!("✅ 购买确认成功！");
            } else {
                let status = response.status();
                let error_text = response.text().unwrap();
                eprintln!("❌ 确认失败: {}", error_text);
                eprintln!("HTTP {}: {}", status, error_text);
            }
        }
        Err(e) => {
            println!("❌ Payment verification failed: {:?}", e);
            // 错误处理...
        }
    }
}
