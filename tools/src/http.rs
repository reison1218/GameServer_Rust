use super::*;
use async_h1::client;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::{Arc as AsyncArc, RwLock as AsyncRwLock};
use async_std::task;
use http_types::{Body, Error as HttpTypesError, Method, Request, Response, StatusCode, Url};
use serde::export::Result::Err;
use serde_json::{Error, Value};

pub trait HttpServerHandler: Send + Sync {
    fn get_path(&self) -> &str;
    fn execute(
        &mut self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, HttpTypesError>;
}

pub async fn http_server(
    address: &str,
    handler_vec: Vec<Box<dyn HttpServerHandler>>,
) -> http_types::Result<()> {
    // Open up a TCP connection and create a URL.
    //let listener = TcpListener::bind(("127.0.0.1", 8080)).await?;
    let listener = TcpListener::bind(address).await?;
    let addr = format!("http://{}", listener.local_addr()?);
    info!("HTTP-SERVER listening on {}", addr);

    let handler_vec_arc = AsyncArc::new(AsyncRwLock::new(handler_vec));
    // For each incoming TCP connection, spawn a task and call `accept`.
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let addr = addr.clone();
        let h_c = handler_vec_arc.clone();
        task::spawn(async move {
            if let Err(err) = accept(addr, stream, h_c).await {
                eprintln!("{}", err);
            }
        });
    }
    Ok(())
}

// Take a TCP stream, and convert it into sequential HTTP request / response pairs.
async fn accept(
    addr: String,
    stream: TcpStream,
    handler_vec: AsyncArc<AsyncRwLock<Vec<Box<dyn HttpServerHandler>>>>,
) -> http_types::Result<()> {
    info!(
        "there is new connection from {}",
        stream.peer_addr().unwrap()
    );
    async_h1::accept(addr.as_str(),stream.clone(), |mut _req| async {
        let mut _req = _req;
        let mut _req_mut = &mut _req;
        _req_mut
            .insert_header("Content-Type", "application/json")
            .unwrap();
        info!(
            "receive a http request from:{:?}",
            stream.peer_addr().unwrap()
        );
        //获取body和其中的参数，并将参数转换成serde_json::value结构体
        let mut body = _req_mut.take_body();
        let body_len = body.len();
        let mut params: Option<Value> = None;
        match body_len {
            Some(len) => {
                if len > 0 {
                    let mut string = String::new();
                    body.read_to_string(&mut string).await.unwrap();
                    let json: Result<serde_json::Value, Error> =
                        serde_json::from_str(string.as_str());
                    if json.is_err() {
                        error!("http server error!{:?}", json.as_ref().err().unwrap());
                        return Err(http_types::Error::from_str(
                            StatusCode::BadRequest,
                            json.err().unwrap().to_string(),
                        ));
                    }
                    params = Some(json.unwrap());
                }
            }
            None => {}
        }
        std::mem::drop(_req_mut);

        //获取path
        let mut _req_mut = &mut _req;
        let mut path_segments = _req_mut
            .url()
            .path_segments()
            .ok_or_else(|| "cannot be base")
            .unwrap();
        let path = path_segments.next().unwrap();
        let mut write = handler_vec.write().await;
        //开始遍历path进行过滤
        let iter = write.iter_mut();
        let mut result: Option<Result<serde_json::Value, http_types::Error>> = None;
        for handler in iter {
            if !handler.get_path().eq(path) {
                continue;
            }
            result = Some(handler.execute(params));
            break;
        }

        if result.is_none() {
            error!("{:?}", "result is none!");
            return Err(http_types::Error::from_str(
                StatusCode::NoContent,
                "result is none!",
            ));
        }
        if result.as_mut().unwrap().is_err() {
            error!("{:?}", result.as_mut().unwrap().as_mut().err().unwrap());
            return Err(result.unwrap().err().unwrap());
        }
        if result.as_mut().unwrap().as_mut().is_err() {
            error!("{:?}", result.as_mut().unwrap().as_mut().err().unwrap());
            return Err(http_types::Error::from_str(
                StatusCode::NoContent,
                "unwrap has err!",
            ));
        }

        //设置返回参数
        let mut res = Response::new(StatusCode::Ok);
        //res.insert_header("Content-Type", "text/plain")?;
        res.insert_header("Content-Type", "application/json")
            .unwrap();
        res.set_body(result.unwrap().unwrap().to_string());
        Ok(res)
    })
    .await?;
    Ok(())
}

///发送http请求
#[warn(unused_assignments)]
pub async fn send_http_request(
    ip_port: &str,
    path: &str,
    method: &str,
    params: Option<Value>,
) -> Result<Value, HttpTypesError> {
    let http_method: Option<Method>;
    match method {
        "post" => http_method = Some(Method::Post),
        "get" => http_method = Some(Method::Get),
        _ => http_method = Some(Method::Post),
    }
    let stream = TcpStream::connect(ip_port).await?;
    let peer_addr = stream.peer_addr()?;
    let str = format!("http://{}/{}", peer_addr, path);
    info!("connecting to {:?}", str.as_str());
    let url = Url::parse(str.as_str())?;
    let mut req = Request::new(http_method.unwrap(), url);
    req.insert_header("Content-Type", "application/json");
    match params {
        Some(p) => {
            req.set_body(Body::from(p.to_string()));
        }
        None => {}
    }

    let mut res = client::connect(stream.clone(), req).await?;
    if !res.status().is_success() {
        let status = res.status();
        let error_str = format!("http request fail!,error:{:?}", status.to_string());
        return Err(HttpTypesError::from_str(status, error_str));
    }
    let mut body = res.take_body();
    let  result: Option<Value>;
    match body.len() {
        Some(len) => {
            if len <= 0 {
                return Err(HttpTypesError::from_str(
                    StatusCode::NoContent,
                    "body is empty!",
                ));
            }
            let mut string = String::new();
            body.read_to_string(&mut string).await.unwrap();
            let json: serde_json::Value = serde_json::from_str(string.as_str())?;
            result = Some(json)
        }
        None => {
            return Err(HttpTypesError::from_str(
                StatusCode::NoContent,
                "body is empty!",
            ));
        }
    }

    Ok(result.unwrap())
}
