mod sig_rec;
mod tcp_client;
mod test_async;
mod test_tokio;
mod web;
mod web_socket;

use log::info;
use num_enum::FromPrimitive;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use protobuf::Message;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::json;
use slab::Slab;
use std::time::{Duration, Instant, SystemTime};
use tools::http::HttpMethod;
use tools::protos::base::{RankInfoPt, WorldCellPt};

//use tcp::thread_pool::{MyThreadPool, ThreadPoolHandler};
// use tcp::tcp::ClientHandler;
// use tcp::util::bytebuf::ByteBuf;
// use tcp::util::packet::Packet;

use std::collections::{BinaryHeap, HashMap, HashSet, LinkedList};
use std::sync::mpsc::{channel, Receiver};

//use tokio::net::{TcpListener as TokioTcpListener,TcpStream as TokioTcpStream};
//use tokio::prelude::*;
//use tokio::runtime::Runtime as TokioRuntime;
//use tokio::net::tcp::{ReadHalf,WriteHalf};
use std::error::Error;
//use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use async_std::io;
use async_std::net::{TcpListener as AsyncTcpListener, TcpStream as AsyncTcpStream};
use async_std::prelude::*;
use async_std::task;

use crate::web::test_http_client;
use crate::web::{test_faster, test_http_server};
use actix::{Actor, ContextFutureSpawner, SyncArbiter};
use chrono::{Datelike, Local, Timelike};
use crossbeam::atomic::{AtomicCell, AtomicConsume};
use crossbeam::sync::ShardedLock;
use envmnt::{ExpandOptions, ExpansionType};
use futures::executor::block_on;
use futures::future::join3;
use futures::SinkExt;
use futures::{join, FutureExt};
use rand::prelude::*;
use rayon::prelude::ParallelSliceMut;
use serde_json::Value;
use std::alloc::System;
use std::any::Any;
use std::borrow::{Borrow, BorrowMut, Cow};
use std::cell::{Cell, RefCell, RefMut};
use std::collections::binary_heap::PeekMut;
use std::collections::btree_map::Entry::Vacant;
use std::collections::btree_map::Range;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Debug, Display};
use std::fs::File;
use std::hash::Hasher;
use std::io::{Read, Write};
use std::mem::Discriminant;
use std::ops::{Deref, DerefMut};
use std::panic::catch_unwind;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{JoinHandle, Thread};
use std::{env, ptr};
use threadpool::ThreadPool;
use tools::macros::GetMutRef;
use tools::protos::room::C_LEAVE_ROOM;
use tools::redis_pool::RedisPoolTool;
use tools::tcp::ClientHandler;
use tools::templates::template::{init_temps_mgr, TemplatesMgr};
use tools::util::bytebuf::ByteBuf;
use tools::util::packet::Packet;

#[macro_use]
extern crate lazy_static;

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
        ["a", hh @ ..] => println!("ends with: {:?}", hh),

        rest => println!("{:?}", rest),
    }
}

fn test_tcp_client() {
    for i in 0..=1 {
        let m = move || {
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

fn test_binary() {
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
#[derive(Debug, Clone, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
enum HH {
    AA = 1,
}

struct TT {
    s: String,
}

impl PartialEq for TT {
    fn eq(&self, other: &Self) -> bool {
        self.s.eq_ignore_ascii_case(other.s.as_str())
    }
}

impl std::cmp::Eq for TT {}

impl std::hash::Hash for TT {
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

#[derive(Debug, Default)]
struct Foo {
    x: i32,
    y: String,
}

impl Foo {
    pub fn get_x(&self) -> i32 {
        self.x
    }
}

impl Deref for Foo {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.y
    }
}

#[derive(Default)]
struct BaseFoo {
    foo: Option<Foo>,
}

fn test_err() -> anyhow::Result<()> {
    anyhow::bail!("test");
}

#[derive(Debug)]
pub struct Form<T: Sized> {
    p: T,
}

impl<T> Form<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.p
    }
}

trait Layoutable {
    fn position(&self) -> (f32, f32);
    fn size(&self) -> (f32, f32);
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
struct TestMacro {
    pos: (f32, f32),
    size: (f32, f32),
}

impl_layoutable!(TestMacro);

trait DoSomething<T> {
    fn do_sth(&self, value: T);
}
impl<'a, T: Debug> DoSomething<T> for &'a usize {
    fn do_sth(&self, value: T) {
        println!("{:?}", value);
    }
}

// fn do_foo<'a>(b:Box<dyn DoSomething<&'a usize>>){
//     let s:usize = 10;
//     b.do_sth(&s);
// }

fn do_bar(b: Box<dyn for<'f> DoSomething<&'f usize>>) {
    let s: usize = 10;
    b.do_sth(&s);
}

pub fn test_str<'b, 'a: 'b>(str: &'a str, str1: &'a str) -> &'b str {
    if str.len() > str1.len() {
        return str;
    }
    str1
}

thread_local! {
    pub static I:Cell<u32> = Cell::new(1);
}

#[derive(Debug)]
pub struct TestT {
    temp: &'static String,
}

impl Drop for TestT {
    fn drop(&mut self) {
        println!("drop Test");
    }
}

pub fn test_unsafe2() {
    let mut t: Option<TestT> = None;
    unsafe {
        let str = String::from("哈哈");
        let str_ptr = str.borrow() as *const String;
        t = Some(TestT {
            temp: str_ptr.as_ref().unwrap(),
        });
    }
    println!("res {:?}", t.as_ref().unwrap().temp);
}

#[derive(Default)]
pub struct TestSize {
    a: u32,
    b: u32,
    c: u32,
}
#[cfg(feature = "bar")]
mod bar {
    pub fn bar() {
        println!("test");
    }
}

#[cfg(any(bar))]
mod ss {
    pub fn test() {
        println!("test");
    }
}

#[derive(Default, Debug, Clone)]
struct STest {
    str: String,
    v: Vec<String>,
}

#[derive(Default)]
struct TestS {
    a: AtomicCell<u32>,
    b: AtomicCell<u32>,
    c: AtomicCell<u32>,
    d: String,
    e: Vec<u32>,
    f: Test,
    g: HashMap<u32, Test>,
}

impl TestS {
    pub fn test(&mut self) {}
}

tools::get_mut_ref!(TestS);

fn calc_n(n: i64) {
    print!("N={},", n);
    let mut ans = 0i64;
    let (mut r1, mut r2, mut r3, mut r4, mut r5, mut r6);
    let now = std::time::Instant::now();
    for a1 in 1..n >> 3 {
        r1 = n - a1;
        for a2 in a1..r1 / 7 {
            r2 = r1 - a2;
            for a3 in a2..r2 / 6 {
                r3 = r2 - a3;
                for a4 in a3..r3 / 5 {
                    r4 = r3 - a4;
                    for a5 in a4..r4 >> 2 {
                        r5 = r4 - a5;
                        for a6 in a5..r5 / 3 {
                            r6 = r5 - a6;
                            for a7 in a6..r6 >> 1 {
                                ans += a1 ^ a2 ^ a3 ^ a4 ^ a5 ^ a6 ^ a7 ^ (r6 - a7);
                            }
                        }
                    }
                }
            }
        }
    }
    println!("{}, cost={:?}", ans, now.elapsed());
}

#[allow(dead_code)]
fn calc_n2(n: i64) {
    print!("N={},", n);
    let mut ans = 0i64;
    let (mut r1, mut r2, mut r3, mut r4, mut r5, mut r6) = (0, 0, 0, 0, 0, 0);
    let now = std::time::Instant::now();
    for a1 in 1..n >> 3 {
        r1 = n - a1;
        for a2 in a1..r1 / 7 {
            r2 = r1 - a2;
            for a3 in a2..r2 / 6 {
                r3 = r2 - a3;
                for a4 in a3..r3 / 5 {
                    r4 = r3 - a4;
                    for a5 in a4..r4 >> 2 {
                        r5 = r4 - a5;

                        (a5..r5 / 3).for_each(|a6| {
                            r6 = r5 - a6;
                            (a6..r6 >> 1).for_each(|a7| {
                                ans += a1 ^ a2 ^ a3 ^ a4 ^ a5 ^ a6 ^ a7 ^ (r6 - a7);
                            });
                        });
                    }
                }
            }
        }
    }
    println!("{}, cost={:?}", ans, now.elapsed());
}

#[derive(Default)]
pub struct PinTest {
    str: String,
    s: Option<&'static mut str>,
}
impl TestTrait for PinTest {}
#[derive(Default)]
pub struct OtherStruct;

impl TestTrait for OtherStruct {}
pub trait TestTrait {
    fn test() {}
}

#[derive(Default, Debug)]
pub struct StructTest {
    id: u32,
    rank: u32,
    str: String,
}

impl Drop for StructTest {
    fn drop(&mut self) {
        println!("drop");
    }
}

impl TestTrait for StructTest {}

pub struct TestDrop {
    s: StructTest,
}

#[derive(Default, Debug)]
pub struct ZZ<T: TestTrait = StructTest> {
    e: T,
}

#[derive(Debug)]
pub struct SSSSSSS<T, const N: usize>([T; N]);

use std::sync::Once;

static START: Once = Once::new();

static mut STATIC_U32: u32 = 0;

pub struct StructTestPtr(*mut StructTest);

fn abc<'b, 'a: 'b>(mut st: &'b mut StructTest, st1: &'a mut StructTest) {
    println!("l_st:{:p}", st);
    st = st1;
    println!("l_st:{:p}", st);
}

pub struct TcpClientTest;
use async_trait::async_trait;
#[async_trait]
impl tools::tcp::ClientHandler for TcpClientTest {
    async fn on_open(&mut self, sender: crossbeam::channel::Sender<Vec<u8>>) {
        println!("open");
    }

    async fn on_close(&mut self) {
        println!("close");
    }

    async fn on_message(&mut self, mess: Vec<u8>) {
        println!("message");
    }
}

fn main() -> anyhow::Result<()> {
    let res = [0, 0, 0, 0];

    fn aaaaaaaaa(res: [u32; 4]) -> u32 {
        let mut index = res.len() - 1;

        loop {
            let a = res[index];
            if a != 0 {
                return a;
            }
            if index == 0 {
                return a;
            }
            index -= 1;
        }
        0
    }
    let res = aaaaaaaaa(res);
    println!("{}", res);

    // let res = std::net::TcpStream::connect("spiritle.test.fabled-game.com:16801");
    // let res = std::net::TcpStream::connect("127.0.0.1:16801");
    // match res {
    //     Ok(mut ts) => {
    //         println!("success");
    //         let s = String::from_str("hello").unwrap();
    //         loop {
    //             let res = ts.write(s.as_bytes());
    //             std::thread::sleep(Duration::from_secs(1));
    //             match res {
    //                 Ok(size) => {
    //                     println!("{}", size);
    //                 }
    //                 Err(err) => {
    //                     println!("{:?}", err);
    //                 }
    //             }
    //         }
    //     }
    //     Err(e) => {
    //         println!("{:?}", e);
    //     }
    // }

    // let mut st = StructTest::default();
    // let mut st1 = StructTest::default();
    // let a = &mut st;
    // let b = &mut st1;
    // println!("w_st:{:p}", a);
    // println!("w_st1:{:p}", b);
    // abc(a, b);
    // println!("w_st:{:p}", a);
    // let res = SSSSSSS([0, 10]);
    // let res1 = SSSSSSS([0, 40]);
    // println!("{:?},{:?}", res.type_id(), res.0.type_id());
    // println!("{:?},{:?}", res1.type_id(), res1.0.type_id());
    // let a = std::ptr::addr_of!(res);
    // unsafe {
    //     println!("{:?}", a.read_unaligned());
    // }

    // START.call_once(|| {
    //     println!("test Once!");
    //     unsafe {
    //         STATIC_U32 = 1;
    //     }
    // });

    // START.call_once(|| {
    //     println!("test Once!");
    // });
    // let mut v = vec![StructTest::default(), StructTest::default()];
    // {
    //     let size = v.len();
    //     let mut res = v.drain(0..size);
    //     let next1 = res.next();
    //     let next2 = res.next();
    //     println!("{:?}", next1);
    //     println!("{:?}", next2);
    //     std::mem::forget(res);
    // }
    // println!("{:?}", v);
    // let s = serde_json::Value::try_from(1).unwrap();
    // s.as_f64()
    // let v:Vec<Box<dyn Send+Sync+'static>> = Vec::new();
    // println!("{}", std::mem::size_of::<ZZ>());
    // let StructTest{a,..} = StructTest::default();
    // println!{"{}",a};
    // calc_n2(50);
    // let res = tools::http::send_http_request("192.168.2.103:7777","reload_temps","post",None);
    // async_std::task::block_on(res);
    //calc_n2(600);
    // let mut tt = TestS::default();
    // let t = tt.borrow_mut();
    // t.g.insert(1,Test::default());
    // let d:&mut String = t.d.borrow_mut();
    // let e:&mut Vec<u32> = t.e.borrow_mut();
    // let f = t.f.borrow_mut();
    // d.push_str("1");
    // f.str.push_str("1");
    // e.push(1);
    // let ttt=  t.g.get_mut(&1).unwrap();
    // ttt.str.push_str("1");

    // let mut arc=  Arc::new(RwLock::new(TestS::default()));
    // for i in 0..9999{
    //     let res = arc.clone();
    //     let m = move||{
    //         let read = res.read().unwrap();
    //     };
    //
    // }
    //  test_unsafe();
    // let mut t = Test::default();
    // t.str.push_str("asdf");
    // t.i.fetch_add(1);
    // unsafe{
    //     let res:Test = std::mem::transmute_copy(&t);
    //     dbg!(res);
    //     dbg!(t);
    // }

    // rc1.borrow().borrow_mut().str.push_str("1");
    // rc2.borrow().borrow_mut().str.push_str("1");
    // tcp_client::test_tcp_client("reison1");
    // crate::bar::bar();
    // crate::ss::test();

    // let rc = RefCell::new(Test::default());
    // rc.borrow_mut().str.push_str("1");

    // let mut s1 = STest::default();
    // s1.str.push_str("s1");
    // s1.v.push("s1".to_owned());
    // let mut s2 = STest::default();
    // s2.str.push_str("s2");
    // s2.v.push("s2".to_owned());
    //
    //
    // std::mem::swap(&mut s1,&mut s2);
    // let (sender,rec) = crossbeam::channel::unbounded();
    // let res = async move {
    //     let time = std::time::SystemTime::now();
    //     for _ in 0..999999999{
    //         sender.send(1);
    //     }
    //     println!("send task time:{:?}",time.elapsed().unwrap());
    // };
    // async_std::task::spawn(res);
    //
    // let time = std::time::SystemTime::now();
    // let mut res:i32 = 0;
    // loop{
    //     res += rec.recv().unwrap();
    //     if res>=999999999{
    //         break;
    //     }
    // }
    // println!("rec task time:{:?},{}",time.elapsed().unwrap(),res);
    // tcp_client::test_tcp_client("reison");
    // let test = async_std::sync::Arc::new(async_std::sync::Mutex::new(Test::default()));
    // let res = async move{
    //     let t = test.clone();
    //     let s = async move{
    //         let lock = t.lock().await;
    //         dbg!("s:{:?}",std::thread::current().name().unwrap());
    //         ()
    //     };
    //
    //     let t1 = test.clone();
    //     let s1 = async move{
    //         let lock = t1.lock().await;
    //         dbg!("s1:{:?}",std::thread::current().name().unwrap());
    //         ()
    //     };
    //     let t2 = test.clone();
    //     let s2 = async move{
    //         let lock = t2.lock().await;
    //         dbg!("s2:{:?}",std::thread::current().name().unwrap());
    //         ()
    //     };
    //     let res = join3(s,s1,s2);
    //     task::spawn(res).await;
    //     ()
    // };
    //
    // async fn test_mutex(name:&str,test:Arc<Mutex<Test>>){
    //     let lock = test.lock().unwrap();
    //     dbg!("{:?}:{:?}",name,std::thread::current().name().unwrap());
    // }
    //
    // block_on(res);
    //
    // std::thread::sleep(Duration::from_millis(50000));
    //
    // test_channel_and_mutex();
    // test_channel();
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

fn test_loop() {
    let mut index = 1_i32;
    'out: loop {
        println!("start");
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            println!("{}", index);
            index += 1;
            if index == 3 {
                index = 1_i32;
                continue 'out;
            }
        }
    }
}

fn test_drop() {
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

fn test_channel_and_mutex() {
    let test = Test::default();
    let arc = Arc::new(tokio::sync::RwLock::new(test));
    let metux_time = std::time::SystemTime::now();
    let mut size = 0;
    loop {
        size += 1;
        if size == 99999 {
            break;
        }
        let arc_clone = arc.clone();
        let m = async move {
            let mut lock = arc_clone.write().await;
            lock.i.fetch_add(1);
            ()
        };
        // std::thread::spawn(m);
        let res = async_std::task::spawn(m);
        block_on(res);
    }
    // let mut builder = std::thread::Builder::new();
    // let res = builder.spawn(||{std::thread::current()}).unwrap();
    // res.join();
    println!(
        "mutex time:{:?},value:{}",
        metux_time.elapsed().unwrap(),
        block_on(arc.write()).i.load()
    );
    // println!("mutex time:{:?},value:{}",metux_time.elapsed().unwrap(),arc.write().await.i.load());

    // let res = async move{
    //     // async_std::task::sleep(Duration::from_millis(1000)).await;
    //     let lock = arc.write().await;
    //     println!("mutex time:{:?},value:{}",metux_time.elapsed().unwrap(),lock.i.load());
    // };
    // async_std::task::spawn(res);

    let (cb_sender, cb_rec) = crossbeam::channel::bounded(102400);
    let m = move || {
        let mut size = 0;
        let rec_time = std::time::SystemTime::now();
        loop {
            let res = cb_rec.recv();
            if let Err(e) = res {
                println!("{:?}", e);
                break;
            }
            size += 1;
            if size == 99999 {
                println!("cb_rec time:{:?}", rec_time.elapsed().unwrap());
            }
        }
    };
    std::thread::spawn(m);
    let send_time = std::time::SystemTime::now();
    for i in 0..99999 {
        cb_sender.send(Test::default());
    }
    println!("cb_send time:{:?}", send_time.elapsed().unwrap());

    std::thread::sleep(Duration::from_millis(50000));
}

fn test_channel() {
    let (std_sender, std_rec) = std::sync::mpsc::sync_channel(102400);
    let m = move || {
        let mut size = 0;
        let rec_time = std::time::SystemTime::now();
        loop {
            let res = std_rec.recv().unwrap();
            size += 1;
            if size == 9999999 {
                println!("std_rec time:{:?}", rec_time.elapsed().unwrap());
            }
        }
    };
    std::thread::spawn(m);
    let send_time = std::time::SystemTime::now();
    for i in 0..9999999 {
        std_sender.send(Test::default());
    }
    println!("std_send time:{:?}", send_time.elapsed().unwrap());

    let (cb_sender, cb_rec) = crossbeam::channel::bounded(102400);

    let m = move || {
        let mut size = 0;
        let rec_time = std::time::SystemTime::now();
        loop {
            let res = cb_rec.recv().unwrap();
            size += 1;
            if size == 9999999 {
                println!("cb_rec time:{:?}", rec_time.elapsed().unwrap());
            }
        }
    };
    std::thread::spawn(m);
    let send_time = std::time::SystemTime::now();
    for i in 0..9999999 {
        cb_sender.send(Test::default());
    }
    println!("cb_send time:{:?}", send_time.elapsed().unwrap());

    std::thread::sleep(Duration::from_millis(5000));
}

#[derive(Debug, Default)]
struct Test {
    pub str: String,
    pub i: AtomicCell<u32>,
}

impl Drop for Test {
    fn drop(&mut self) {
        println!("drop Test");
    }
}

unsafe impl Send for Test {}

unsafe impl Sync for Test {}

fn test_unsafe() {
    unsafe {
        let mut t = Test::default();
        let mut t2: Test = std::mem::transmute_copy(&t);
        t2.i.store(100);
        dbg!(t);
        dbg!(t2);
        // let mut test = Test{str:"test".to_owned(),i:AtomicCell::new(0)};
        // let test_p = &mut test as *mut Test;
        // let s = test_p.as_mut().unwrap();
        // let s1 = test_p.as_mut().unwrap();
        // s1.str.push_str("2");
        // s.str.push_str("1");
        // println!("{:?}",s);
        // println!("{:?}",s1);
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
fn test_sort() {
    let mut v = Vec::new();
    let mut rng = thread_rng();
    for i in 1..=99999 {
        let n: u32 = rng.gen_range(1..99999);
        v.push(n);
    }

    let time = SystemTime::now();
    for i in 1..=9999 {
        v.par_sort_by(|a, b| b.cmp(a));
    }
    //println!("{:?}",v);
    println!("rayon:{:?}", time.elapsed().unwrap());

    let mut v = Vec::new();
    let mut rng = thread_rng();
    for i in 1..=99999 {
        let n: u32 = rng.gen_range(1..99999);
        v.push(n);
    }
    let time = SystemTime::now();
    for i in 1..=9999 {
        v.sort_by(|a, b| b.cmp(a));
    }
    //println!("{:?}",v);
    println!("comment:{:?}", time.elapsed().unwrap());
}

fn test() -> impl Display {
    let res = "test".to_string();
    res
}
