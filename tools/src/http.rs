use std::collections::HashMap;

use super::*;
use axum::{
    body::Bytes,
    extract::Query,
    http::{HeaderMap, HeaderValue},
    routing::{get, post},
};
use serde_json::json;

pub trait HttpServerHandler: Send + Sync {
    fn get_path(&self) -> &str;
    fn do_post(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
        _json_params: &[u8],
    ) -> anyhow::Result<serde_json::Value> {
        Ok(json!(r#"{"code":200,"mess":"success"}"#))
    }
    fn do_get(
        &mut self,
        _uri: String,
        _uri_params: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        Ok(r#"{"statue":200,"mess":"success"}"#.to_string())
    }
}

///axum server builder
pub struct Builder {
    app: axum::Router,
}

impl Builder {
    ///create new axum server size
    pub fn new() -> Self {
        let app = axum::Router::new();
        Builder { app }
    }

    ///add route for axum server
    pub fn route(mut self, handler: Box<dyn HttpServerHandler>) -> Self {

        let handler = std::sync::Arc::new(async_lock::Mutex::new(handler));

        let handler_lock = handler.lock_blocking();

        let path = handler_lock.get_path().to_owned();

        let handler_lock = handler.clone();
        let do_post_c = |uri: axum::http::Uri,
                         Query(uri_params): Query<HashMap<String, String>>,
                         body: Bytes| async move {
            let body_res = body.to_vec();
            //axum::Json(json_params): axum::Json<serde_json::Value>
            let mut headers = HeaderMap::new();
            headers.insert(
                "Content-Type",
                HeaderValue::from_str("text/html;application/json;charset=utf-8").unwrap(),
            );
            headers.insert(
                "Access-Control-Allow-Origin",
                HeaderValue::from_str("*").unwrap(),
            );
            headers.insert("Accept", HeaderValue::from_str("*/*").unwrap());
            let mut handler_lock = handler_lock.lock_blocking();
            //receive the http request
            let res = handler_lock.do_post(uri.to_string(), uri_params, body_res.as_slice());
            drop(handler_lock);
            if let Err(e) = res {
                error!("{:?}", e);
                return (
                    axum::http::StatusCode::PRECONDITION_FAILED,
                    headers,
                    axum::Json(json!({"code":-100,"mess":"fail"})),
                );
            }
            (
                axum::http::StatusCode::OK,
                headers,
                axum::Json(res.unwrap()),
            )
        };

        let handler_lock = handler.clone();
        let do_get_c = |uri: axum::http::Uri, Query(uri_params): Query<HashMap<String, String>>| async move {
            let mut headers = HeaderMap::new();
            headers.insert(
                "Content-Type",
                HeaderValue::from_str("text/html;application/json;charset=utf-8").unwrap(),
            );
            headers.insert(
                "Access-Control-Allow-Origin",
                HeaderValue::from_str("*").unwrap(),
            );

            let mut handler_lock = handler_lock.lock_blocking();
            //receive the http request
            let res = handler_lock.do_get(uri.to_string(), uri_params);
            drop(handler_lock);
            if let Err(e) = res {
                error!("{:?}", e);
                return (
                    axum::http::StatusCode::PRECONDITION_FAILED,
                    headers,
                    r#"{"code":-100,"mess":"fail"}"#.to_string(),
                );
            }
            (axum::http::StatusCode::OK, headers, res.unwrap())
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            HeaderValue::from_str("text/html;application/json;charset=utf-8").unwrap(),
        );
        self.app = self.app.route(path.as_str(), get(do_get_c));
        self.app = self.app.route(path.as_str(), post(do_post_c));
        self
    }

    ///bind ip and listening the port
    pub fn bind(self, port: u16) {
        let m = async move {
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
            axum::Server::bind(&addr)
                .serve(self.app.into_make_service())
                .await
                .unwrap();
        };
        TOKIO_RT.spawn(m);
        info!("http-server listening on {:?}:{}", "0.0.0.0", port);
    }
}

pub fn send_post(url: &str, json: Option<serde_json::Value>) -> anyhow::Result<String> {
    let res = match json {
        Some(json) => ureq::post(url).send_json(json)?.into_string()?,
        None => ureq::post(url).call()?.into_string()?,
    };
    Ok(res)
}

pub fn send_get(
    url: &str,
    url_params: Option<HashMap<String, String>>,
    json: Option<serde_json::Value>,
) -> anyhow::Result<String> {
    let url_res;

    match url_params {
        Some(params) => {
            let mut str = String::new();
            str.push_str(url);
            str.push_str("?");
            str.push_str(map_url(params).as_str());
            url_res = str;
        }
        None => url_res = url.to_string(),
    }
    let res = match json {
        Some(json) => ureq::get(url_res.as_str()).send_json(json)?.into_string()?,
        None => ureq::get(url_res.as_str()).call()?.into_string()?,
    };
    Ok(res)
}

fn map_url(map: HashMap<String, String>) -> String {
    let mut s = String::new();
    let mut index = 0;
    let size = map.len();
    map.iter().for_each(|(key, value)| {
        s.push_str(key);
        s.push_str("=");
        s.push_str(value);
        if index != size - 1 {
            s.push_str("&");
        }
        index += 1;
    });
    s
}
