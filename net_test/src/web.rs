use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use http_types::{Error as HttpTypesError,Body, Url, Method, Request,Response, StatusCode};
use std::ops::Index;
use async_h1::client;
use serde_json::{Error, Map, Value};
use serde_json::value::Value as JsonValue;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use crate::Test;

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
    map.insert("phone_no".to_owned(), JsonValue::from("1231312414"));
    map.insert("platform_id".to_owned(), JsonValue::from(pid.to_owned()));
    let value = JsonValue::from(map);

    let mut body = Body::from(value.to_string());
    req.set_body(body);

    let mut res = client::connect(stream.clone(), req).await?;

    let mut str = String::new();
    res.take_body().read_to_string(&mut str).await.unwrap();
    let mut res_json = Value::from_str(str.as_str()).unwrap();
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
    async_h1::accept(addr.as_str(),stream.clone(), |mut _req| async move {
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
// async fn new_tokio_client(mut stream:TokioTcpStream){
//     let (mut read,mut write) = stream.split();
//     let read_s = async move{
//         println!("start write");
//         let mut bytes:[u8;1024] = [0;1024];
//         loop{
//             let size = read.read(&mut bytes[..]).await.unwrap();
//             println!("{:?}",&bytes[..]);
//         }
//     };
//     let write_s = async move{
//         println!("start write");
//         let mut bytes_w:[u8;1024] = [0;1024];
//         write.write(&mut bytes_w);
//         write.flush();
//     };
//     tokio::task::spawn(read_s);
//     tokio::task::spawn(write_s);
//     println!("new client!");
// }

// fn test_tokio(){
//     let mut runtime = TokioRuntime::new().unwrap();
//     let tcp_server = async{
//         let mut listener = TokioTcpListener::bind("127.0.0.1:8080").await.unwrap();
//         while let Some(stream) = listener.next().await {
//             match stream {
//                 Ok(mut stream) => {
//                     stream.set_recv_buffer_size(1024*32 as usize);
//                     stream.set_send_buffer_size(1024*32 as usize);
//                     stream.set_linger(Some(Duration::from_secs(5)));
//                     stream.set_keepalive(Some(Duration::from_secs(3600)));
//                     stream.set_nodelay(true);
//                     new_tokio_client(stream);
//                     println!("new client!");
//                 },
//                 Err(e) => { /* connection failed */ }
//             }
//         }
//     };
//     runtime.block_on(tcp_server);
// }


//
pub fn test_faster(){

    let (sender,rec) = std::sync::mpsc::sync_channel(1024000);

    let time = std::time::SystemTime::now();
    for _ in 0..1000
    {
        let sender_cp = sender.clone();
        let m = move ||{
            for _ in 0..1000{
                sender_cp.send(1);
            }
        };
        std::thread::spawn(m);
    }

    let mut i = 1;
    loop{
        let res = rec.recv();
        i += res.unwrap();
        if i >= 1000000{
            break;
        }
    }

    println!("channel:{}ms,{}",time.elapsed().unwrap().as_millis(),i);

    let time = std::time::SystemTime::now();
    let test=Arc::new(RwLock::new(Test::default()));
    for _ in 0..1000{
        let t_cp = test.clone();
        let m = move ||{
            for _ in 0..1000{
                let _ = t_cp.write().unwrap().i;
                t_cp.write().unwrap().i+=1;
            }
        };
        std::thread::spawn(m);
    }
    println!("thread:{}ms,{}",time.elapsed().unwrap().as_millis(),test.write().unwrap().i);
}