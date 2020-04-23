use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use http_types::{Error as HttpTypesError,Body, Url, Method, Request,Response, StatusCode};
use std::ops::Index;
use async_h1::client;
use serde_json::Error;

pub async fn test_http_client()->Result<(), HttpTypesError>{
    let stream = TcpStream::connect("localhost:8080").await?;
    let peer_addr = stream.peer_addr()?;
    println!("connecting to {}", peer_addr);

    for i in 0usize..2 {
        println!("making request {}/2", i + 1);
        let url = Url::parse(&format!("http://{}/save", peer_addr)).unwrap();
        let req = Request::new(Method::Post, url);
        let res = client::connect(stream.clone(), req).await?;
        println!("{:?}", res);
    }
    Ok(())
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
