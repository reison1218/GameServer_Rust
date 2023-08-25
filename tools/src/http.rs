use std::collections::HashMap;

use super::*;
use async_std::sync::{Arc as AsyncArc, RwLock as AsyncRwLock};
use serde_json::json;

pub enum HttpMethod {
    POST,
    GET,
}

pub trait HttpServerHandler: Send + Sync {
    fn get_path(&self) -> &str;
    fn get_method(&self) -> HttpMethod;
    fn do_post(&mut self, _: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        Ok(json!(r#"{"statue","success"}"#))
    }
    fn do_get(&mut self, _: HashMap<&str, &str>) -> anyhow::Result<String> {
        Ok(r#"{"statue","success"}"#.to_string())
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
        let handler = AsyncArc::new(AsyncRwLock::new(handler));

        let handler_lock = TOKIO_RT.block_on(handler.write());

        let http_methond = handler_lock.get_method();

        let path = handler_lock.get_path().to_owned();

        std::mem::drop(handler_lock);

        let handler_lock = handler.clone();
        let do_post_c = |axum::Json(params): axum::Json<serde_json::Value>| async move {
            let mut handler_lock = handler_lock.write().await;
            //receive the http request
            let res = handler_lock.do_post(params);
            drop(handler_lock);
            if let Err(e) = res {
                error!("{:?}", e);
                return (
                    axum::http::StatusCode::PRECONDITION_FAILED,
                    axum::Json(json!({"result":"fail"})),
                );
            }

            (axum::http::StatusCode::CREATED, axum::Json(res.unwrap()))
        };

        let handler_lock = handler.clone();
        let do_get_c = |uri: axum::http::Uri| async move {
            let mut handler_lock = handler_lock.write().await;
            let query = uri.query();
            if query.is_none() {
                let res = handler_lock.do_get(HashMap::new());
                return (axum::http::StatusCode::CREATED, res.unwrap());
            }
            let query = query.unwrap();
            let v: Vec<&str> = query.split("&").collect();
            let mut map = HashMap::new();

            for _v in v {
                let vec: Vec<&str> = _v.split("=").collect();
                map.insert(
                    vec.get(0).unwrap().to_owned(),
                    vec.get(1).unwrap().to_owned(),
                );
            }
            //receive the http request
            let res = handler_lock.do_get(map);
            drop(handler_lock);
            if let Err(e) = res {
                error!("{:?}", e);
                return (
                    axum::http::StatusCode::PRECONDITION_FAILED,
                    r#"{"result":"fail"}"#.to_string(),
                );
            }
            (axum::http::StatusCode::CREATED, res.unwrap())
        };
        let http_methond: axum::routing::MethodRouter = match http_methond {
            HttpMethod::POST => axum::routing::post(do_post_c),
            HttpMethod::GET => axum::routing::get(do_get_c),
        };
        self.app = self.app.route(path.as_str(), http_methond);
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
        Some(json) => ureq::post(url).send_json(json).unwrap().into_string()?,
        None => ureq::post(url).call().unwrap().into_string()?,
    };
    Ok(res)
}

pub fn send_get(
    url: &str,
    url_params: Option<HashMap<&str, &str>>,
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

fn map_url(map: HashMap<&str, &str>) -> String {
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
