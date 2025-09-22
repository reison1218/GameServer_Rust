pub mod pay;
pub mod login;

use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{anyhow, Context};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};



// JWT Claims
#[derive(Debug, Serialize)]
struct Claims {
    iss: String,   // 服务账号邮箱
    scope: String, // 请求的权限范围
    aud: String,   // 令牌端点
    exp: i64,      // 过期时间
    iat: i64,      // 签发时间
}


// 服务账号凭据
#[derive(Debug, Deserialize)]
pub struct ServiceAccountCredentials {
    #[serde(rename = "private_key")]
    private_key: String,
    #[serde(rename = "client_email")]
    client_email: String,
    #[serde(rename = "token_uri")]
    token_uri: String,
}


// 令牌响应
#[derive(Debug, Deserialize)]
pub struct GoogleApiToken {
    pub access_token: String, //令牌
    pub token_type: String,   //token_type
    pub expires_in: i64,      //过期时间
}


//交换API令牌
pub fn exchange_token(service_account_path: &str) -> anyhow::Result<GoogleApiToken> {
    // 1. 加载服务账号凭据
    let account =
        load_service_account(service_account_path).context("Failed to load service account")?;

    // 2. 获取访问令牌
    let access_token = get_access_token(&account).context("Failed to get access token")?;
    Ok(access_token)
}

/// 从 JSON 文件加载服务账号凭据
fn load_service_account(path: &str) -> anyhow::Result<ServiceAccountCredentials> {
    let content = fs::read_to_string(path)?;
    let account: ServiceAccountCredentials = serde_json::from_str(&content)?;
    Ok(account)
}



/// 获取 Google API 访问令牌
pub fn get_access_token(account: &ServiceAccountCredentials) -> anyhow::Result<GoogleApiToken> {
    let jwt = generate_jwt(account)?;

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&account.token_uri)
        .form(&[
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", &jwt),
        ])
        .send()?;


    if !response.status().is_success() {
        let error_text = response.text()?;
        return Err(anyhow!("Token request failed: {}", error_text));
    }

    let token_response: GoogleApiToken = response.json()?;
    Ok(token_response)
}


/// 生成 JWT 用于获取访问令牌
fn generate_jwt(account: &ServiceAccountCredentials) -> anyhow::Result<String> {
    let now = Utc::now();
    let expiry = now + Duration::hours(1);

    let claims = Claims {
        iss: account.client_email.clone(),
        scope: "https://www.googleapis.com/auth/androidpublisher".to_string(),
        aud: account.token_uri.clone(),
        exp: expiry.timestamp(),
        iat: now.timestamp(),
    };

    let private_key = account.private_key.as_str();
    let key = EncodingKey::from_rsa_pem(private_key.as_bytes())?;
    let header = Header::new(Algorithm::RS256);

    let token = jsonwebtoken::encode(&header, &claims, &key)?;
    Ok(token)
}