mod web;
mod tcp_client;
mod web_socket;
mod mio_test;
use serde_json::json;
use std::time::{Duration, SystemTime};
use protobuf::Message;
//use tcp::thread_pool::{MyThreadPool, ThreadPoolHandler};
// use tcp::tcp::ClientHandler;
// use tcp::util::bytebuf::ByteBuf;
// use tcp::util::packet::Packet;
use futures::executor::block_on;
use std::collections::{HashMap, BinaryHeap, LinkedList};
use std::sync::mpsc::Receiver;

//use tokio::net::{TcpListener as TokioTcpListener,TcpStream as TokioTcpStream};
//use tokio::prelude::*;
//use tokio::runtime::Runtime as TokioRuntime;
//use tokio::net::tcp::{ReadHalf,WriteHalf};
use std::error::Error;
//use std::io::{Read, Write};
use std::net::{TcpStream, TcpListener};

use async_std::io;
use async_std::net::{TcpListener as AsyncTcpListener, TcpStream as AsyncTcpStream};
use async_std::prelude::*;
use async_std::task;


use std::io::{Write, Read};
use tools::tcp::ClientHandler;
use tools::util::packet::Packet;
use std::collections::btree_map::Entry::Vacant;
use std::collections::binary_heap::PeekMut;
use crate::web::test_http_server;
use crate::web::test_http_client;
use threadpool::ThreadPool;
use std::any::Any;
use envmnt::{ExpandOptions, ExpansionType};
use std::ops::DerefMut;
use rand::prelude::*;
use std::collections::BTreeMap;
use std::alloc::System;
use std::cell::{Cell, RefCell};
use serde_json::Value;
use serde::private::de::IdentifierDeserializer;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicU32;
use tools::redis_pool::RedisPoolTool;
use tools::util::bytebuf::ByteBuf;
use std::panic::catch_unwind;
use std::fs::File;
use std::env;

#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref ID:Arc<RwLock<AtomicU32>>={
        let id:Arc<RwLock<AtomicU32>> = Arc::new(RwLock::new(AtomicU32::new(1011000000)));
        id
    };
}
macro_rules! test{
    ($a:expr)=>{
        if $a>0 {
            println!("{}",$a);
        };
    };
}

macro_rules! map{
    (@unit $($x:tt)*) => (());
    (@count $($rest:expr),*)=>(<[()]>::len(&[$(map!(@unit $rest)),*]));
    ($($key:expr=>$value:expr$(,)*)*)=>{
    {
        let cap = map!(@count $($key),*);
        let mut _map = std::collections::HashMap::with_capacity(cap);
        $(
         _map.insert($key,$value);
        )*
        _map
    };
    };
}

fn foo(words: &[&str]) {
    match words {
        // Ignore everything but the last element, which must be "!".
        [.., "!"] => println!("!!!"),

        // `start` is a slice of everything except the last element, which must be "z".
        [start @ .., "z"] => println!("starts with: {:?}", start),

        // `end` is a slice of everything but the first element, which must be "a".
        ["a", hh  @..] => println!("ends with: {:?}", hh),

        rest => println!("{:?}", rest),
    }
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

async fn test_async_std(){
    let mut listener = async_std::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        println!("new client!");
        let mut read_stream = stream.unwrap();
        let mut write_stream = read_stream.clone();
        let read =  async move{
            println!("start read");
            let mut bytes:[u8;1024] = [0;1024];
            loop{
                let size = read_stream.read(&mut bytes).await.unwrap();
                println!("{}",size);
            }
        };

        let write = async move{
            println!("start write");
            let mut bytes:[u8;1024] = [0;1024];
            write_stream.write_all(&bytes[..]);
        };
        async_std::task::spawn(read);
        async_std::task::spawn(write);
    }
}

#[derive(Debug)]
struct TestBox {
    i:u32
}


pub fn max_level_sum(num: RefCell<Option<i32>>) {
    let num = num;

    if let None = *num.borrow() {
        println!("none");
    };
}

fn test(b:&mut TestBox){
    println!("{:p}",b);
    println!("{:?}",b);
}

fn test_channel(){

    let (sender,rec) = std::sync::mpsc::sync_channel(1024000);

    let time = std::time::SystemTime::now();
    for i in 0..1000
    {
        let sender_cp = sender.clone();
        let m = move ||{
            for i in 0..1000{
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
    let test=Arc::new(RwLock::new(Test{i:0}));
    for i in 0..1000{
        let t_cp = test.clone();
        let m = move ||{
            for j in 0..1000{
                let i = t_cp.write().unwrap().i;
                t_cp.write().unwrap().i+=1;
            }
        };
        let j = std::thread::spawn(m);
        j.join();
    }
    let test_cp = test.clone();
    // loop{
    //     let t = test_cp.read().unwrap();
    //     if t.i>=100000{
    println!("thread:{}ms,{}",time.elapsed().unwrap().as_millis(),test.read().unwrap().i);
    //         break;
    //     }
    // }
}

pub struct  Test{
    pub i:u32
}

fn main() -> anyhow::Result<()> {
    //tcp_client::test_tcp_client("platform_id");
    //block_on(web::test_http_client("1"));
    let mut path = env::current_dir()?;
    //path.push("/config");
    let mut str = path.as_os_str().to_str().unwrap();
    let res = str.to_string()+"/config";
    println!("{:?}",res);
    Ok(())
}
