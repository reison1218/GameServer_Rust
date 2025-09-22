use std::error::Error;
use std::str::FromStr;
use std::time::Duration;
use log::info;
use reqwest::{header, Client};
use serde::Deserialize;
use serde::Serialize;
use tools::json::{JsonValue, JsonValueTrait};
use crate::{CLIENT_ID, CLIENT_SECRET, REDIRECT_URI, TOKEN_URL};

// Google API 响应结构
#[derive(Debug, Default,Deserialize,Serialize)]
pub struct GoogleUserTokenInfo {
    pub iss: String,//Token 发行者标识
    pub azp: String,//授权请求的客户端应用 ID
    pub aud: String,// Token 的目标接收者
    pub sub: String,// Google 用户唯一 ID
    pub email: Option<String>,// 用户注册邮箱
    pub email_verified: Option<String>,//邮箱验证状态
    pub at_hash: String,//关联 Access Token 的哈希值
    pub name: Option<String>,//用户全名
    pub picture: Option<String>,//用户头像 URL
    pub given_name: Option<String>,//用户名字（First Name）
    pub family_name: Option<String>,//用户姓氏（Last Name）
    pub iat: String,//Token 签发时间戳 1755487252
    pub exp: String,// Token 过期时间戳
    pub alg: String,//Token 签名算法
    pub kid: String,//签名密钥 ID
    pub typ: String,//Token 类型  JWT 表示此 Token 是 JSON Web Token
}

impl GoogleUserTokenInfo {
    pub fn to_json(&self) -> JsonValue{
        let mut map = JsonValue::new();
        map.insert("iss".to_string(),JsonValue::from(self.iss.clone()));
        map.insert("azp".to_string(),JsonValue::from(self.azp.clone()));
        map.insert("aud".to_string(),JsonValue::from(self.aud.clone()));
        map.insert("sub".to_string(),JsonValue::from(self.sub.clone()));
        map.insert("email".to_string(),JsonValue::from(self.email.clone()));
        map.insert("email_verified".to_string(),JsonValue::from(self.email_verified.clone()));
        map.insert("at_hash".to_string(),JsonValue::from(self.at_hash.clone()));
        map.insert("picture".to_string(),JsonValue::from(self.picture.clone()));
        map.insert("given_name".to_string(),JsonValue::from(self.given_name.clone()));
        map.insert("family_name".to_string(),JsonValue::from(self.family_name.clone()));
        map.insert("iat".to_string(),JsonValue::from(self.iat.clone()));
        map.insert("exp".to_string(),JsonValue::from(self.exp.clone()));
        map.insert("alg".to_string(),JsonValue::from(self.alg.clone()));
        map.insert("kid".to_string(),JsonValue::from(self.kid.clone()));
        map.insert("typ".to_string(),JsonValue::from(self.typ.clone()));
        map
    }
}


// 定义令牌响应的数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub token_type: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

// 这里传验证id的令牌
pub  fn verify_google_id_token(
    token: &str,
) -> anyhow::Result<GoogleUserTokenInfo> {
    let client = reqwest::blocking::Client::new();
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        token
    );

    let response = client.get(&url).send()?;
    if !response.status().is_success() {
        anyhow::bail!("Google API returned error: {}", response.status());
    }

    let token_info: GoogleUserTokenInfo = response.json()?;

    // 验证 token 是否过期（可选）
    let now = chrono::Utc::now().timestamp();
    if i64::from_str(token_info.exp.as_str()).unwrap() < now {
        anyhow::bail!("Token has expired");
    }

    Ok(token_info)
}




// 获取访问令牌的函数
pub fn exchange_code_for_token(
    authorization_code: &str,
) -> Result<TokenResponse, Box<dyn Error>> {
    let params = [
        ("code", authorization_code),
        ("client_id", CLIENT_ID.as_str()),
        ("client_secret", CLIENT_SECRET.as_str()),
        // ("redirect_uri", REDIRECT_URI.as_str()),
        ("grant_type", "authorization_code"),
    ];
    info!("params: {:?}", params);
    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(5)).build()?;
    let response = client
        .post(TOKEN_URL.as_str())
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&params)
        .send()?;

    if !response.status().is_success() {
        let error_text = response.text()?;
        return Err(format!("Error from Google API: token:{},res_mes:{}", authorization_code,error_text).into());
    }

    let json_res = response.json().unwrap();

    let token_response: TokenResponse = json_res;
    Ok(token_response)
}

async fn refresh_token(refresh_token: &str) -> Result<TokenResponse, Box<dyn Error>> {
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token), // 替换为你的刷新令牌
        ("client_id", CLIENT_ID.as_str()),
        ("client_secret", CLIENT_SECRET.as_str()),
    ];
    let client = reqwest::Client::builder().timeout(Duration::from_secs(5)).build()?;
    let response = client
        .post(TOKEN_URL.as_str())
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await?;
    if !response.status().is_success() {
        let error_text = response.text().await?;
        return Err(format!("Error from Google API: {}", error_text).into());
    }

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response)
}