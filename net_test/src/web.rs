use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use http_types::{Error as HttpTypesError,Body, Url, Method, Request,Response, StatusCode};
use std::ops::Index;
use async_h1::client;
use serde_json::{Error, Map, Value};
use futures::TryFutureExt;
use serde_json::value::Value as JsonValue;
use std::str::FromStr;

pub async fn test_http_client(pid:&str)->Result<u32, HttpTypesError>{
    //let stream = TcpStream::connect("localhost:8888").await?;
    let stream = TcpStream::connect("192.168.1.100:8888").await?;
    let peer_addr = stream.peer_addr()?;
    println!("connecting to {}", peer_addr);

    let url = Url::parse(&format!("http://{}/center/user_id", peer_addr)).unwrap();
    let mut req = Request::new(Method::Post, url);
    // let data = r#"
    //     {
    //         "register_platform":"test",
    //         "platform_id": "1",
    //         "game_id": 101,
    //         "nick_name":"test",
    //         "phone_no":"1231312414"
    //     }"#;
    // let mut value = serde_json::Value::from(data);

    let mut map: Map<String, JsonValue> = Map::new();
    map.insert("register_platform".to_owned(), JsonValue::from("test"));
    map.insert("game_id".to_owned(), JsonValue::from(101));
    map.insert("nick_name".to_owned(), JsonValue::from("test"));
    map.insert("avatar".to_owned(), JsonValue::from("test123"));
    map.insert("phone_no".to_owned(), JsonValue::from("1231312414"));
    map.insert("platform_id".to_owned(), JsonValue::from(pid.to_owned()));
    let value = JsonValue::from(map);

    let mut body = Body::from(value.to_string());
    req.set_body(body);

    let mut res = client::connect(stream.clone(), req).await?;

    let mut str = String::new();
    res.take_body().read_to_string(&mut str).await.unwrap();
    println!("{:?}",&str);
    let mut res_json = Value::from_str(str.as_str());
    let mut res_json = res_json.unwrap();
    let map = res_json.as_object_mut().unwrap();
    println!("{:?}", map);
    let user_id = map.get("user_id").unwrap().as_u64().unwrap() as u32;
    Ok(user_id)
}

pub async fn test_http_server() -> http_types::Result<()> {
    // Open up a TCP connection and create a URL.
    let listener = TcpListener::bind(("127.0.0.1", 8080)).await?;
    let addr = format!("http://{}", listener.local_addr()?);
    println!("listening on {}", addr);

    // For each incoming TCP connection, spawn a task and call `accept`.
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let addr = addr.clone();
        task::spawn(async {
            if let Err(err) = accept(addr, stream).await {
                eprintln!("{}", err);
            }
        });
    }
    Ok(())
}

// Take a TCP stream, and convert it into sequential HTTP request / response pairs.
async fn accept(addr: String, stream: TcpStream) -> http_types::Result<()> {
    println!("starting new connection from {}", stream.peer_addr().unwrap());
    async_h1::accept(&addr, stream.clone(), |mut _req| async move {
        _req.insert_header("Content-Type", "application/json").unwrap();
       let url = _req.url();

        //获取path
        let mut path_segments = url.path_segments().ok_or_else(|| "cannot be base").unwrap();
        if "action".eq(path_segments.next().unwrap()){
            println!("action");
        }
        let str= url.query();
        match str {
            None=>{},
            Some(s)=>{println!("{:?}",str);}
        }

        let mut body: Body = _req.take_body();
        let mut string = String::new();
        body.read_to_string(&mut string).await.unwrap();
        println!("{:?}",string);
        let mut json:Result<serde_json::Value,Error> = serde_json::from_str(string.as_str());
        if json.is_err(){
            println!("{:?}",json.as_ref().err().unwrap());
        }else{
            println!("{:?}",json.unwrap());
        }

        //获取参数
        let mut res = Response::new(StatusCode::Ok);
        //res.insert_header("Content-Type", "text/plain")?;
        res.insert_header("Content-Type", "application/json").unwrap();
        res.set_body("Hello");
        Ok(res)
    }).await?;
    Ok(())
}
