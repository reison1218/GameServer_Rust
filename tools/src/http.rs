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
    fn do_get(&mut self, _: HashMap<&str, &str>) -> anyhow::Result<&'static str> {
        Ok(r#"{"statue","success"}"#)
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

            if let Err(e) = res {
                error!("{:?}", e);
                return (
                    axum::http::StatusCode::PRECONDITION_FAILED,
                    r#"{"result":"fail"}"#,
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
            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
            axum::Server::bind(&addr)
                .serve(self.app.into_make_service())
                .await
                .unwrap();
        };
        TOKIO_RT.spawn(m);
        info!("http-server listening on {:?}:{}", "127.0.0,.1", port);
    }
}
