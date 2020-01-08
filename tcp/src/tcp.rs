use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::atomic::{AtomicUsize,Ordering};
use async_std::task;
use std::io::{Write, Read};
use std::sync::mpsc::{channel, Sender, Receiver, SyncSender};
use crate::util::bytebuf::ByteBuf;
use crate::util::packet::{Packet,PacketDes};
use threadpool::ThreadPool;
use std::time;
use simplelog::{
    CombinedLogger, SharedLogger, SimpleLogger, TermLogger, TerminalMode, WriteLogger,
};
use log::{debug, error, info, warn, LevelFilter, Log, Record};

fn init_log() {
    let mut log_time = time::SystemTime::now();
    let mut config = simplelog::ConfigBuilder::new();
    config.set_time_format_str("%Y-%m-%d %H:%M:%S");
    config.set_target_level(LevelFilter::Error);
    config.set_location_level(LevelFilter::Error);
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, config.build(), TerminalMode::Mixed).unwrap(),
    ]).unwrap();
}

pub trait Handler:Send+Sync+Clone{
    fn on_open(&mut self,sender:Sender<Packet>);

    fn on_close(&mut self);

    fn on_message(&mut self,mess:[u8;512]);
}

pub struct TcpChannel{
    tcp:TcpStream,
    id:usize,
}

impl TcpChannel {
    ///写数据
    fn write(&mut self,bytes:&[u8])->usize{
        let size = self.tcp.write(bytes);
        size.unwrap()
    }

    ///写数据，并冲刷
    fn write_and_flush(&mut self,bytes:&[u8])->usize{
        let size = self.tcp.write(bytes);
        self.tcp.flush().unwrap();
        size.unwrap()
    }

    ///获取channel唯一id
    fn get_id(&self)->usize{
        self.id
    }
}

//tcpServer模块
pub mod tcp_server{
    use super::*;
    use std::time::Duration;
    use std::fs::read;
    use std::sync::mpsc::sync_channel;
    use std::borrow::BorrowMut;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use threadpool::ThreadPool;
    use std::ops::{Deref, DerefMut};


    pub struct  TcpServer<T:Handler>{
        server:TcpListener,
        handler:T,
        channel_id:AtomicUsize,
    }

    impl<T:'static> TcpServer<T> where T:Handler{
        ///开始监听
        pub fn on_listen(&mut self){
            let thread_boss = ThreadPool::with_name("tcp-server".to_owned(),4);
            info!("TCP-SERVER:start listening!");
            for stream in self.server.incoming(){
                match stream {
                    Ok(s)=> {
                        info!("there is new client connect:{:?}",s.peer_addr());
                        self.channel_id.store(1, Ordering::Relaxed);
                        let id = self.channel_id.load(Ordering::Relaxed);
                        let mut tcp_channel_read = TcpChannel { tcp: s.try_clone().unwrap(), id: id };

                        let mut tcp_channel_write = TcpChannel { tcp: s, id: id };

                        let mut  handler = self.handler.clone();

                        let (mut sender, mut receiver) = channel();

                        //客户端读取闭包
                        let m = move || {

                            //读取客户端字节
                            let mut read_size: usize = 0;
                            handler.on_open(sender);
                            loop {
                                //先处理读的
                                let mut bytes: [u8; 512] = [0; 512];
                                {
                                    let size = tcp_channel_read.tcp.read(&mut bytes);
                                    if size.is_err() {
                                        error!("TCP-SERVER:{:?}",size.unwrap_err());
                                        handler.on_close();
                                        break;
                                    }
                                    let size = size.unwrap();
                                    if size > 0 {
                                        info!("TCP-SERVER:data of message from client:{:?} ,data size:{}",tcp_channel_read.tcp.peer_addr(),size);
                                        handler.on_message(bytes);
                                    }
                                };
                            };
                        };

                        let m_r = move||{
                            //读取线程管道字节，并用tcpstream写到客户端
                            //再处理从其他线程发过来的写的
                            let d = Duration::from_millis(50);
                            let packet = receiver.recv_timeout(d);
                            if packet.is_ok() {
                                let packet = packet.unwrap();
                                let size = tcp_channel_write.tcp.write(packet.get_data()).unwrap();
                                if size > 0 {
                                    tcp_channel_write.tcp.flush().unwrap();
                                }
                            }
                        };

                        thread_boss.execute(m);
                        thread_boss.execute(m_r);
                    },
                    Err(e)=>{
                        error!("{:?}",e);
                    }
                }
            }
        }
    }


    ///创建TcpServer结构体
    pub fn new<T:Handler>(address : &str,handler:T)->Result<TcpServer<T>,std::io::Error>{
        init_log();
        let tl = TcpListener::bind(address);
        match tl {
            Ok(tl)=>{
                let ato:AtomicUsize = AtomicUsize::new(1000);
                let ts = TcpServer{server:tl,handler:handler,channel_id:ato};
                info!("TCP-SERVER:start bind:{:?}",address);
                Ok(ts)
            },
            Err(e)=>{
                Err(e)
            }
        }
    }
}

pub trait ClientHandler:Send+Sync+Clone{
    fn on_open(&self,ts : &mut TcpStream);

    fn on_close(&mut self);

    fn on_message(&mut self,ts : &mut TcpStream,mess:[u8;512]);
}

///tcpclient模块
pub mod tcp_client{
    use super::*;
    use std::error::Error;
    use std::time::Duration;

    pub struct TcpClient<T:ClientHandler>{
        handler:T,
    }

    impl<T:'static> TcpClient<T> where T:ClientHandler{

        ///创建TcpClient结构体并开始连接
        pub fn new(t:T)->Result<TcpClient<T>,std::io::Error>{
            init_log();
            let mut client = TcpClient{handler:t};
            Ok(client)
        }

        ///读取字节
        pub fn on_read(&mut self,address:&str){

            let thread_boss = ThreadPool::with_name("tcp-client".to_owned(),4);
            //let (mut sender,mut receiver) = channel();
            let mut handler = self.handler.clone();

            let mut read =TcpStream::connect(address).unwrap();

            //let mut write = read.try_clone().unwrap();
            println!("调用try_clone()");
            handler.on_open(&mut read);
            let mut read_bytes:[u8;512] = [0;512];
            info!("start read from {:?}", address);
            loop{
                //读取从tcp服务端发过来的数据
                let size = read.read(&mut read_bytes);
                if size.is_err(){
                    error!("TCP-CLIENT:{:?}",size.unwrap_err());
                    handler.on_close();
                    break;
                }
                let size = size.unwrap();
                //如果读取到的字节数大于0则交给handler
                if size > 0 as usize{
                    info!("TCP-CLIENT:data of message from server {:?} ,data size:{}",address,size);
                    //读取到字节交给handler处理来处理
                    handler.on_message(&mut read,read_bytes);
                }
            }



//            let read_m = move||{
//                handler.on_open(sender);
//                loop{
//                    //读取从tcp服务端发过来的数据
//                    let mut read_bytes:[u8;512] = [0;512];
//                    println!("开始read()");
//                    let size = read.read(&mut read_bytes);
//                    if size.is_err(){
//                        println!("{:?}",size.unwrap_err());
//                        handler.on_close();
//                        break;
//                    }
//                    //如果读取到的字节数大于0则交给handler
//                    if size.unwrap() > 0 as usize{
//                        //读取到字节交给handler处理来处理
//                        handler.on_message(read_bytes);
//                    }
//                }
//            };
//
//            let write_m = move||{
//                loop{
//                    //处理channel写过来的数据
//                    let packet = receiver.recv();
//                    if packet.is_err(){
//                        continue;
//                    }
//                    let packet = packet.unwrap();
//                    let size = write.write(packet.get_data()).unwrap();
//                    if size > 0 {
//                        write.flush().unwrap();
//                    }
//                }
//            };
//            thread_boss.execute(read_m);
//            thread_boss.execute(write_m);
        }
    }
}