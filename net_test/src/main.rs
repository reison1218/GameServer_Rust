mod web;
mod tcp_client;
mod web_socket;
mod mio_test;
mod map;
mod test_async;
mod behavior_test;
mod test_tokio;
use serde_json::json;
use std::time::{Duration, SystemTime, Instant};
use protobuf::Message;
use num_enum::TryFromPrimitive;
use num_enum::IntoPrimitive;
use num_enum::FromPrimitive;
use log::info;


//use tcp::thread_pool::{MyThreadPool, ThreadPoolHandler};
// use tcp::tcp::ClientHandler;
// use tcp::util::bytebuf::ByteBuf;
// use tcp::util::packet::Packet;

use std::collections::{HashMap, BinaryHeap, LinkedList, HashSet};
use std::sync::mpsc::{Receiver, channel};

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
use tools::tcp::{ClientHandler, new_tcp_client};
use tools::util::packet::Packet;
use std::collections::btree_map::Entry::Vacant;
use std::collections::binary_heap::PeekMut;
use crate::web::{test_http_server, test_faster};
use crate::web::test_http_client;
use threadpool::ThreadPool;
use std::any::Any;
use envmnt::{ExpandOptions, ExpansionType};
use std::ops::{DerefMut, Deref};
use rand::prelude::*;
use std::collections::BTreeMap;
use std::alloc::System;
use std::cell::{Cell, RefCell, RefMut};
use serde_json::Value;
use serde::private::de::IdentifierDeserializer;
use std::str::FromStr;
use std::sync::{Arc, RwLock, Mutex, Condvar};
use std::sync::atomic::AtomicU32;
use tools::redis_pool::RedisPoolTool;
use tools::util::bytebuf::ByteBuf;
use std::panic::catch_unwind;
use std::fs::File;
use std::env;
use chrono::{Local, Datelike, Timelike};
use std::fmt::{Display, Debug};
use std::mem::Discriminant;
use futures::executor::block_on;
use std::thread::{Thread, JoinHandle};
use rayon::prelude::ParallelSliceMut;
use futures::SinkExt;
use std::borrow::{Borrow, BorrowMut};
use std::hash::Hasher;
use std::rc::Rc;
use futures::join;
use crate::test_async::async_main;
use std::collections::btree_map::Range;
use tools::templates::template::{init_temps_mgr, TemplatesMgr};
use crate::map::generate_map;
use actix::{Actor, SyncArbiter};
use std::convert::TryInto;
use crossbeam::atomic::{AtomicConsume, AtomicCell};
use tools::macros::GetMutRef;

#[macro_use]
extern crate lazy_static;
extern crate proc_macro;

lazy_static! {
    static ref ID:Arc<RwLock<AtomicU32>>={
        let id:Arc<RwLock<AtomicU32>> = Arc::new(RwLock::new(AtomicU32::new(1011000025)));
        id
    };

    ///静态配置文件
    static ref TEMPLATES: TemplatesMgr = {
        let path = env::current_dir().unwrap();
        let str = path.as_os_str().to_str().unwrap();
        let res = str.to_string()+"/template";
        let conf = init_temps_mgr(res.as_str());
        conf
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

fn test_tcp_client(){
    for i in 0..=1{
        let m = move ||{
            let mut str = "test".to_owned();
            str.push_str(i.to_string().as_str());
            tcp_client::test_tcp_client(str.as_str());
        };
        std::thread::spawn(m);
        std::thread::sleep(Duration::from_millis(2000));
    }
     //std::thread::sleep(Duration::from_millis(40000));
    tcp_client::test_tcp_client("test");
}

fn test_binary(){
    // let int = 123u32;
    // //(1)最原始直接基础的位操作方法。
    // let mut byte: u8 = 0b0000_0000;
    // println!("{:0x}", int);
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
}



// macro_rules! test{
//
//     ($key:expr=>$value:expr,$yunsuan:ident)=>{
//         if $key  $yunsuan $value{
//             true
//         }else{
//         false
//         }
//     };
// }

// {
// "panding": {
// "cell_type": 1,
// "yunsuanfu": ">",
// "canshu": 1
// },
// "result":{"true":[1001,1002],"false":[1004]}
// }
#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive,IntoPrimitive)]
#[repr(u8)]
enum  HH{
    AA=1,
}

struct  TT{
    s:String,
}

impl PartialEq for TT{
    fn eq(&self, other: &Self) -> bool {
        self.s.eq_ignore_ascii_case(other.s.as_str())
    }
}

impl std::cmp::Eq for TT{

}

impl std::hash::Hash for TT{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.s.hash(state)
    }
}



// impl std::cmp::PartialEq<HH> for TT{
//     fn eq(&self, other: &HH) -> bool {
//         self.s == *other
//     }
// }

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

#[derive(Debug,Default)]
struct Foo {
    x: i32,
    y:String,
}

impl  Foo{
    pub fn get_x(&self)->i32{
        self.x
    }
}

impl Deref for Foo{

    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.y
    }
}


#[derive(Default)]
struct BaseFoo{
    foo:Option<Foo>
}

fn test_err()->anyhow::Result<()>{
    anyhow::bail!("test");
}

#[derive(Debug)]
pub struct Form<T:Sized>{p: T}

impl<T> Form<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.p
    }
}

trait Layoutable {
    fn position(&self) -> (f32,f32);
    fn size(&self) -> (f32,f32);
    fn set_position(&mut self, x: f32, y: f32);
    fn set_size(&mut self, width: f32, height: f32);
}
macro_rules! impl_layoutable {
    ($e: ty) => {
        impl Layoutable for $e {
            fn position(&self) -> (f32,f32) { self.pos }
            fn size(&self) -> (f32,f32) { self.size }
            fn set_position(&mut self, x: f32, y: f32) { self.pos = (x, y); }
            fn set_size(&mut self, width: f32, height: f32) { self.size = (width, height); }
        }
    };
}

#[derive(Default)]
struct TestMacro{
pos: (f32, f32),
size: (f32, f32)
}

impl_layoutable!(TestMacro);


#[derive(Default)]
pub struct TTT{
    s:crossbeam::atomic::AtomicCell<String>,
    d:crossbeam::atomic::AtomicCell<Vec<u8>>,
}


pub struct TestLift{
    pub str:&'static String,
}

impl Drop for TestLift{
    fn drop(&mut self) {
        dbg!("drop TestLift");
    }
}

impl tools::macros::GetMutRef for TestLift{}



trait DoSomething<T>{
    fn do_sth(&self,value:T);
}
impl <'a,T:Debug> DoSomething<T> for &'a usize{
    fn do_sth(&self, value: T) {
        println!("{:?}",value);
    }
}

// fn do_foo<'a>(b:Box<dyn DoSomething<&'a usize>>){
//     let s:usize = 10;
//     b.do_sth(&s);
// }

fn do_bar(b:Box<dyn for<'f> DoSomething<&'f usize>>){
    let s:usize = 10;
    b.do_sth(&s);
}


fn main() -> anyhow::Result<()> {

    //test_channel();
    // let x = Box::new(&2usize);
    // do_bar(x);

    // And set a new one
    //hostname::set("potato")?;


    // let s: String = "Hello, World".to_string();
    // let any: Box<dyn Any> = Box::new(s);
    // let res:Box<String> = any.downcast().unwrap();
    // dbg!(res);
    // let t = TTT::default();
    // let res = t.d.take();
    // println!("{:?}", t.borrow().d.take());
    //test_faster();
    //tcp_client::test_tcp_client("reison");
    // let m = move||{
    //     loop{
    //
    //     }
    // };
    // std::thread::spawn(m);
    // let builder = std::thread::Builder::new();
    // // handler.join().unwrap();
    //
    // let handler = builder
    //     .spawn(|| {
    //         std::thread::current()
    //     })
    //     .unwrap();
    // handler.join().expect("Couldn't join on the associated thread");


    // let sleep_time = res.timestamp() - date.timestamp();
    // println!("{}",sleep_time);
    //let test = TestMacro::default();
    // let mut map= HashMap::new();
    // map.insert(1,Rc::new(RefCell::new(Form{p:String::new()})));
    // let res = map.get_mut(&1).unwrap();
    // let mut re = Cell::new(Form{p:String::new()});

    // println!("{:?}",v);
    // println!("{:?}",res);
    //tcp_client::test_tcp_clients();
    // let season_temp = TEMPLATES.get_season_temp_mgr_ref().get_temp(&1001).unwrap();
     //map::generate_map();
    // let a:u8 = HH::AA.into();
    // println!("{}",a)
    // let words:[u32;5] = [1,2,3,4,5];
    //
    // let id = 2_u32;
    // match id {
    //     ss@ =>{
    //
    //     }
    // }
    // let a = -1;
    // let b = 2;
    // let res = b+*&a;
    // println!("{}",res);
    // let mut k =&&&&&Foo{x:10,y:String::from("test")};
    // print!("{}",k.get_x());
    // println!("{:?}",k.bytes());

    //generate_map();
    // let foo = Foo{x:1};
    // let mut rc = Rc::new(foo);
    //
    //
    // block_on(async_main());



    //test_unsafe();
    //
    // let mut foo = Foo { x: 42 };
    //
    // let x = &mut foo.x;
    // *x = 13;
    // let y = foo;
    // println!("{:?}", (&y).x);  //only added this line
    // println!("{:?}", y.x); //13

    //let test = test!(1=>2,<);
    //crate::map::generate_map();
    // let i:u8 = 0;
    // let j = true;
    // println!("{}",std::mem::size_of_val(&i));
    // println!("{}",std::mem::size_of_val(&j));
    //test_binary();
    //test_sort();
    //test_tcp_client();
    //map::generate_map();
    // let res = Local::now().timestamp_millis();
    // println!("{}",res);
    //test_channel();
    //test_loop();
    Ok(())
}

fn test_loop(){
    let mut index = 1_i32;
    'out:loop{
        println!("start");
        loop{
            std::thread::sleep(Duration::from_millis(1000));
            println!("{}",index);
            index+=1;
            if index == 3{
                index = 1_i32;
                continue 'out;
            }
        }
    }
}

fn test_drop(){
    {
        let _a = Count(3);
        let _ = Count(2);
        let _c = Count(1);
    }
    {
        let _a = Count(3);
        let _b = Count(2);
        let _c = Count(1);
    }
}

struct Count(i32);

impl Drop for Count {
    fn drop(&mut self) {
        println!("dropping count {}", self.0);
    }
}


fn test_channel(){
    let (std_sender,std_rec) = std::sync::mpsc::sync_channel(102400);
    let m = move||{
        let mut size = 0;
        let rec_time = std::time::SystemTime::now();
      loop{
          let res = std_rec.recv().unwrap();
          size+=1;
          if size == 9999999{
              println!("std_rec time:{:?}",rec_time.elapsed().unwrap());
          }
      }
    };
    std::thread::spawn(m);
    let send_time = std::time::SystemTime::now();
    for i in 0..9999999{
        std_sender.send(Test::default());
    }
    println!("std_send time:{:?}",send_time.elapsed().unwrap());

    let (cb_sender,cb_rec) = crossbeam::channel::bounded(102400);

    let m = move||{
        let mut size = 0;
        let rec_time = std::time::SystemTime::now();
        loop{
            let res = cb_rec.recv().unwrap();
            size+=1;
            if size == 9999999{
                println!("cb_rec time:{:?}",rec_time.elapsed().unwrap());
            }
        }
    };
    std::thread::spawn(m);
    let send_time = std::time::SystemTime::now();
    for i in 0..9999999{
        cb_sender.send(Test::default());
    }
    println!("cb_send time:{:?}",send_time.elapsed().unwrap());

    std::thread::sleep(Duration::from_millis(5000));

}



#[derive(Debug,Default)]
struct Test{
    pub str:String,
    pub i:u32,
}

fn test_unsafe(){
    unsafe {
        let mut test = Test{str:"test".to_owned(),i:0};
        let test_p = &mut test as *mut Test;
        let s = test_p.as_mut().unwrap();
        let s1 = test_p.as_mut().unwrap();
        s1.str.push_str("2");
        s.str.push_str("1");
        println!("{:?}",s);
        println!("{:?}",s1);
        // let mut str = "test".to_owned();
        // let s_p = &str as *const String;
        // let s_p_m = &mut str as *mut String;
        // assert_eq!(s_p, s_p_m);
        // println!("s_p:{}", *s_p);
        // println!("s_p_m:{}", *s_p_m);
        // std::mem::drop(str);
        // let s_p_m = &mut *s_p_m;
        // s_p_m.push_str("sss");
        // println!("str:{:?}", s_p_m);
        //
        // let address = 0x7ffee3b103af_usize;
        // let s = address as *mut String;
        // println!("{:?}",s);
        // let s = &mut *s;
        // s.push_str("ss");
        // println!("{:?}",s);
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
    for i in 1..=9999{
        v.par_sort_by(|a,b|b.cmp(a));
    }
    //println!("{:?}",v);
    println!("rayon:{:?}",time.elapsed().unwrap());

    let mut v = Vec::new();
    let mut rng = thread_rng();
    for i in 1..=99999{
        let n: u32 = rng.gen_range(1, 99999);
        v.push(n);
    }
    let time = SystemTime::now();
    for i in 1..=9999{
        v.sort_by(|a,b|b.cmp(a));
    }
    //println!("{:?}",v);
    println!("comment:{:?}",time.elapsed().unwrap());
}


fn test()->impl Display{
    let res = "test".to_string();
    res
}