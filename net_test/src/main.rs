mod message_io;
mod sig_rec;
mod tcp_client;
mod test_async;
mod test_tokio;
mod web;
mod web_socket;

use ::message_io::network::Transport;
use bma_benchmark::benchmark;
use futures::future::Fuse;
use futures::select;
use futures::Future;
use futures::FutureExt;
use log::info;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::task::Poll;
use std::time::{Duration, SystemTime};
use tools::net_message_io::MessageHandler;
use tools::net_message_io::TransportWay;
use tools::tcp::ClientHandler;
use tracing_subscriber::EnvFilter;

//use tcp::thread_pool::{MyThreadPool, ThreadPoolHandler};
// use tcp::tcp::ClientHandler;
// use tcp::util::bytebuf::ByteBuf;
// use tcp::util::packet::Packet;

use std::collections::HashMap;

//use tokio::net::{TcpListener as TokioTcpListener,TcpStream as TokioTcpStream};
//use tokio::prelude::*;
//use tokio::runtime::Runtime as TokioRuntime;
//use tokio::net::tcp::{ReadHalf,WriteHalf};
//use std::io::{Read, Write};

use crate::web::test_http_client;
use crossbeam::atomic::AtomicCell;
use futures::executor::block_on;
use rand::prelude::*;
use rayon::prelude::ParallelSliceMut;
use std::borrow::Borrow;
use std::cell::Cell;
use std::env;
use std::fmt::{Debug, Display};
use std::hash::Hasher;
use std::ops::Deref;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, RwLock};

use tools::templates::template::{init_temps_mgr, TemplatesMgr};

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

pub fn abcd<T: Debug + 'static>(str: T) {
    let mut map = HashMap::new();
    map.insert("1".to_owned(), str);
    map.remove("1");
    println!("{:?}", map.get("1").unwrap());
}

fn test_close(mut a: impl FnMut(u32)) {
    a(1);
}

#[derive(Clone)]
pub struct MessageClient;
use async_trait::async_trait;
#[async_trait]
impl tools::net_message_io::MessageHandler for MessageClient {
    async fn try_clone(&self) -> Self {
        todo!()
    }

    async fn on_open(&mut self, tcp_handler: tools::net_message_io::NetHandler) {
        println!("链接上了");
        let str = String::from_str("ss").unwrap();
        tcp_handler.send(str.as_bytes());
        unsafe {
            client_num.fetch_add(1, Ordering::SeqCst);

            println!("{}", client_num.load(Ordering::SeqCst));
        }
    }

    async fn on_close(&mut self) {
        println!("断开了");
        self.connect(TransportWay::Tcp, "127.0.0.1:16801").await;
    }

    async fn on_message(&mut self, mess: &[u8]) {}
}

#[derive(Default)]
pub struct TcpClientTestt {
    pub sender: Option<crossbeam::channel::Sender<Vec<u8>>>,
}
#[async_trait]
impl tools::tcp::ClientHandler for TcpClientTestt {
    async fn on_open(&mut self, sender: crossbeam::channel::Sender<Vec<u8>>) {
        println!("连上了");
        let str = String::from_str("ss").unwrap();
        sender.send(str.as_bytes().to_vec());
        client_num.fetch_add(1, Ordering::SeqCst);

        println!("{}", client_num.load(Ordering::SeqCst));
    }

    async fn on_close(&mut self) {
        println!("断开了");
        self.on_read("127.0.0.1:16801".to_string()).await;
    }

    async fn on_message(&mut self, mess: Vec<u8>) {}
}

static client_num: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

pub struct DumbFuture {
    a: String,
    b: *const String,
}

impl DumbFuture {
    fn init(&mut self) {
        let self_ref: *const String = &self.a;
        self.b = self_ref;
    }
    fn a(&self) -> &str {
        &self.a
    }

    fn b(&self) -> &String {
        assert!(
            !self.b.is_null(),
            "Test::b called without Test::init being called first"
        );
        unsafe { &*(self.b) }
    }
}

#[derive(Default)]
pub struct Rust {
    a: String,
    b: u16,
    c: u8,
}

pub fn find_median_sorted_arrays(nums1: Vec<i32>, nums2: Vec<i32>) -> f64 {
    let mut array = nums1;
    array.extend_from_slice(nums2.as_slice());
    array.sort();
    if array.len() >= 3 {
        array.remove(0);
        array.pop();
    }

    let res: i32 = array.iter().sum();
    let res = res as f64;
    let len = array.len() as f64;
    res / len
}

extern "C" {
    fn rand() -> u32;
}

fn main() -> anyhow::Result<()> {
    let m = 1;
    print!("{}", m);

    // let res = find_median_sorted_arrays(Vec::from([1, 2, 3]), Vec::from([4, 5, 6]));
    // println!("{}", res);
    // Print some basic info about the response to standard output.
    // println!("Status: {}", response.status());
    // println!("Headers: {:#?}", response.headers());

    // Read the response body as text into a string and print it.

    // let mut test1 = DumbFuture {
    //     a: String::from_str("test1").unwrap(),
    //     b: std::ptr::null(),
    // };
    // test1.init();
    // let mut test2 = DumbFuture {
    //     a: String::from_str("test2").unwrap(),
    //     b: std::ptr::null(),
    // };
    // test2.init();
    // println!("test1-a: {}, b: {}", test1.a(), test1.b());
    // println!("test2-a: {}, b: {}", test2.a(), test2.b());

    // println!("------------------------------------------------");
    // println!("test1-a: {}, b: {}", test1.a(), test1.b());
    // std::mem::swap(&mut test1, &mut test2);
    // println!("test2-a: {}, b: {}", test2.a(), test2.b());

    // let a = DumbFuture;
    // async_std::task::block_on(a);
    // let mut mc = MessageClient;
    // async_std::task::block_on(mc.connect(TransportWay::Tcp, "127.0.0.1:16801"));
    // let mut tct = TcpClientTestt::default();
    // async_std::task::block_on(tct.on_read("127.0.0.1:16801".to_string()));
    // test_close(move |x| {
    //     println!("{}", x);
    // });
    // let mut tcp = std::net::TcpStream::connect("localhost:16801").unwrap();
    // let mut bytes: [u8; 512] = [0; 512];
    // loop {
    //     let res = tcp.read(&mut bytes);
    //     let size = res.unwrap();
    //     println!("{}", size);
    //     if size == 0 {
    //         break;
    //     }
    // }
    // println!("over");
    // let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3042);
    // let m = move || {
    //     std::thread::sleep(Duration::from_secs(10));
    //     message_io::run(Transport::Tcp, socket);
    // };
    // std::thread::spawn(m);
    // std::thread::sleep(Duration::from_micros(1));
    // let mut tcp = std::net::TcpStream::connect("localhost:16888").unwrap();
    // let buf = "hello".as_bytes();
    // tcp.write(buf);
    // tcp.flush();
    // loop {
    //     std::thread::sleep(Duration::from_micros(2));
    //     let mut bytes: [u8; 128] = [0; 128];
    //     let size = tcp.read(&mut bytes);
    //     if let Ok(size) = size {
    //         if size > 0 {
    //             println!("client Received: {}", String::from_utf8_lossy(&bytes));
    //         } else {
    //             println!("client disconnect");
    //             break;
    //         }
    //     }
    //     tcp.write(buf);
    //     tcp.flush();
    // }
    // std::thread::sleep(Duration::from_micros(10));

    // for _ in 0..100 {
    //     let stream = std::net::TcpStream::connect("localhost:16801").unwrap();
    // }
    // tcp_client::test_tcp_client("reison2");

    // let ticket="140000002122985e271b14182b7e180001001001c1d4b06018000000010000000200000055a1c4a738e3c606c029070001000000b200000032000000040000002b7e180001001001067d18003cb20fb77302a8c000000000ed6ca4606d1cc060010005af080000000000c5379ac1ac49f4b5488c02e5a327a2759d52a8da892f1d649c69745a8a530d6b3ad1128a6864db03eb5a7de7c30562c822ac646886091bdbe0c6cf5629266d06e4898dee90bcadf139ceb73103b5a694f17fae162b2d5971b2734cc3acf88f9e76a4767e7c4c156666d6f54e1c9d9a2dc8fa7d9d2454a0dbe94ee7f73f0cd9c2";

    // let  url = format!("https://partner.steam-api.com/ISteamUserAuth/AuthenticateUserTicket/v1/?key={:?}&appid={}&ticket={:?}","DC8AD15E088033860FD8C08C02591AFD",1604870,ticket);

    // let url = url.replace(r#"""#, "");
    // let res = String::new();
    // let mut request = http::Request::builder()
    //     .uri(url.as_str())
    //     .header("User-Agent", "awesome/1.0")
    //     .body(res)
    //     .unwrap();
    // println!("{:?}", res);
    // let resp = reqwest::blocking::get(url.as_str())?;

    // let res = resp.text().unwrap();
    // println!("{:#?}", res);

    // let url = format!("https://partner.steam-api.com/ISteamUser/CheckAppOwnership/v2/?key={:?}&appid={}&steamid={:?}","DC8AD15E088033860FD8C08C02591AFD",1604870,ticket);

    // let url = format!("https://partner.steam-api.com/ISteamUser/CheckAppOwnership/v2/?key={:?}&appid={}&steamid={:?}","DC8AD15E088033860FD8C08C02591AFD",1604870,ticket);
    // let url = url.replace(r#"""#, "");
    // use http::Request;
    // let mut body = String::new();
    // let request = Request::builder()
    //     .uri(url.as_str())
    //     .header("User-Agent", "awesome/1.0")
    //     .body(body)
    //     .unwrap();
    // println!("{:?}", request);
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
