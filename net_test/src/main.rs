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
use std::sync::{Arc, RwLock, Mutex};
use std::sync::atomic::AtomicU32;
use tools::redis_pool::RedisPoolTool;
use tools::util::bytebuf::ByteBuf;
use std::panic::catch_unwind;
use std::fs::File;
use std::env;
use chrono::Local;
use std::fmt::Display;
use std::mem::Discriminant;
use futures::executor::block_on;
use std::thread::Thread;
use rayon::prelude::ParallelSliceMut;


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

fn main() -> anyhow::Result<()> {
    for i in 0..=2{
        let m = move ||{
            let mut str = "test".to_owned();
            str.push_str(i.to_string().as_str());
            tcp_client::test_tcp_client(str.as_str());
        };
        std::thread::spawn(m);
        std::thread::sleep(Duration::from_millis(2000));
    }
    tcp_client::test_tcp_client("test");

    // block_on(http);
    // print!("http执行完毕");
    //block_on(web::test_http_client("1"));
    //block_on(web::test_http_server());
    // let mut path = env::current_dir()?;
    // path.push("/config");
    // let mut str = path.as_os_str().to_str().unwrap();
    //let res = str.to_string()+"/config";
    //println!("{:?}",res);


    // let int = 123u32;
    // //(1)最原始直接基础的位操作方法。
    // let mut byte: u8 = 0b0000_0000;
    // println!("{:0b}", int);
    // byte |= 0b0000_1000; // Set a bit
    // println!("0b{:08b}", byte);
    // byte &= 0b1111_0111; // Unset a bit
    // println!("0b{:08b}", byte);
    // byte ^= 0b0000_1000; // Toggle a bit
    // println!("0b{:08b}", byte);
    // byte = !byte; // Flip all bits
    // println!("0b{:08b}", byte);
    // byte <<= 1; // shift left one bit
    // println!("0b{:08b}", byte);
    // byte >>= 1; // shift right one bit
    // println!("0b{:08b}", byte);
    // //特别提醒：rust为每一个数字类型都实现了大量方法，其中包括位操作方法！！！具体请参看下方链接！！！
    // //https://doc.rust-lang.org/std/primitive.u8.html
    // let mut rbyte: u8 = 0b1000_0000;
    // rbyte = rbyte.rotate_left(1); // rotate left one bit
    // println!("0b{:08b}", byte);
    // //https://doc.rust-lang.org/std/#primitives
    // rbyte = rbyte.rotate_right(1); // rotate right one bit
    // println!("0b{:08b}", rbyte);
    // bit_twiddling(0, 3);
    // bit_twiddling(8, 3);
    //test bitwise operation macros
    // assert_eq!(eq1!(0b0000_1111, 0), true);
    // assert_eq!(eq0!(0b0000_1111, 4), true);
    // assert_eq!(set!(0b0000_1111, 0), 0x0f);
    // assert_eq!(clr!(0b0000_1111, 0), 0x0e);

    // for i in 1..999999{
    //     let m = move ||{
    //       loop{
    //           std::thread::sleep(Duration::from_millis(60000));
    //       }
    //     };
    //     let thread = std::thread::spawn(m);
    //
    //     println!("{}",i);
    // }
    //test_sort();
    Ok(())
}

#[derive(Debug)]
struct Test{
    str:String
}


async fn async_test(){
    println!("test");
}

fn test_unsafe(){
    unsafe {
        let mut str = "test".to_owned();
        let s_p = &str as *const String;
        let s_p_m = &mut str as *mut String;
        assert_eq!(s_p, s_p_m);
        println!("s_p:{}", *s_p);
        println!("s_p_m:{}", *s_p_m);
        std::mem::drop(str);
        let s_p_m = &mut *s_p_m;
        s_p_m.push_str("sss");
        println!("str:{:?}", s_p_m);

        let address = 0x7ffee3b103af_usize;
        let s = address as *mut String;
        println!("{:?}",s);
        let s = &mut *s;
        s.push_str("ss");
        println!("{:?}",s);
    }
}
fn test_sort(){
    let mut v = Vec::new();
    let mut rng = thread_rng();
    for i in 1..=99999{
        let n: u32 = rng.gen_range(1, 99999);
        v.push(n);
    }

    let time = SystemTime::now();
    for i in 1..10{
        v.par_sort_by(|a,b|b.cmp(a));
    }
    //println!("{:?}",v);
    println!("rayon:{}",time.elapsed().unwrap().as_millis());

    let mut v = Vec::new();
    let mut rng = thread_rng();
    for i in 1..=99999{
        let n: u32 = rng.gen_range(1, 99999);
        v.push(n);
    }
    let time = SystemTime::now();
    for i in 1..10{
        v.sort_by(|a,b|b.cmp(a));
    }
    //println!("{:?}",v);
    println!("comment:{}",time.elapsed().unwrap().as_millis());
}


fn test()->impl Display{
    let res = "test".to_string();
    res
}