use super::*;
use crate::util::packet::{Packet};
use async_std::task;
use simplelog::{
    CombinedLogger,TermLogger, TerminalMode,
};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, Sender};
use std::time;
use std::time::Duration;
use std::io::ErrorKind;
use std::net::Shutdown;
use std::sync::mpsc::RecvTimeoutError;
use net2::TcpStreamExt;
use std::sync::Arc;
use futures::SinkExt;
use std::sync::mpsc::SendError;
use mio::Token;
use std::marker::{Send,Sync};


///The TCP server side handler is used to handle TCP general events, such as connections,
/// closing connections, having data transfers
pub trait Handler: Send + Sync{
    fn try_clone(&self)->Self;
    ///Triggered when there is a new client connection
    fn on_open(&mut self, sender:TcpSender);

    ///Disconnect triggered when client was closed
    fn on_close(&mut self);

    ///Triggered when there is client data transfer
    fn on_message(&mut self, mess: Vec<u8>);
}

///tcp server sender
#[derive(Clone)]
pub struct TcpSender{
    pub sender:SyncSender<Data>,
    pub token:usize,
}

impl TcpSender {
    pub fn write(&mut self, bytes: Vec<u8>) -> Result<(),SendError<Data>> {
        self.sender.send(Data{bytes,token:self.token})
    }

    fn get_token(&self) -> usize {
        self.token
    }
}

/// for TCP hanler end channel to transfer data
#[derive(Clone)]
pub struct Data{
    pub token:usize,
    pub bytes:Vec<u8>
}

unsafe impl Send for Data{}
unsafe impl Sync for Data{}

///TCP server module, just need impl Handler, and call the new function, can run the TCP server program,
/// each client corresponds to a separate handler for client requests. The following is an example.
/// like this:
/// ```
/// #[derive(Clone)]
/// struct ServerHandler{
///     test:u32,//You can put some Pointers or some data that you need
/// }
///
/// impl Handler for ServerHanddler{
///     fn try_clone(&self) -> Self {
///         self.clone();//Here, implement the clone function yourself, or add #[derive(clone)] to the ServerHandler as above
///     }
///     ///当有新客户端链接当时候调用
///     fn on_open(&mut self, sender: TcpSender) {
///         //do something here what u need
///      }
///     ///当客户端断开链接当时候调用
///     fn on_close(&mut self) {
///         //do something here what u need
///     }
///     ///当客户端发消息过来的时候调用
///     fn on_message(&mut self, mess: Vec<u8>) {
///         //do something here what u need
///     }
/// }
/// ```
///
pub mod tcp_server {
    use super::*;
    use futures::executor::block_on;
    use std::sync::mpsc::sync_channel;
    use std::sync::{Arc, RwLock};
    use std::time::Duration;
    use threadpool::ThreadPool;
    use std::convert::TryInto;
    use std::io::{BufReader, BufRead};
    use mio::net::{TcpListener as MioTcpListener, TcpStream as MioTcpStream};
    use mio::{Events, Interest, Poll, Registry, Token};
    use mio::event::{Event, Source};
    use std::io::{self, Read, Write};
    use std::collections::hash_map::HashMap;
    use std::str::{from_utf8, FromStr};
    use std::net::SocketAddr;
    use std::rc::Rc;
    use std::cell::{RefCell, Cell};
    use std::borrow::BorrowMut;
    use std::error::Error;
    use futures::{AsyncWriteExt, TryFutureExt};

    ///事件的唯一标示
    const SERVER: Token = Token(0);

    ///Create the TCP server and start listening on the port
    pub fn new<T: Handler>(addr: &str, mut handler: T)->io::Result<()> {
        // Create a poll instance.
        let mut poll = Poll::new()?;
        // Create storage for events.
        let mut events = Events::with_capacity(512);
        // tcp监听地址
        let address = SocketAddr::from_str(addr).unwrap();
        // Setup the TCP server socket.
        let mut server = MioTcpListener::bind(address)?;

        // Map of `Token` -> `TcpStream`.
        let mut conn_map = Arc::new(RwLock::new(HashMap::new()));
        //handlermap
        let mut handler_map = HashMap::new();
        // Unique token for each incoming connection.
        let mut unique_token = Token(SERVER.0 + 1);
        //异步消息管道，用来接收所有handler的sender消息
        let (sender,rec) = std::sync::mpsc::sync_channel(102400);
        //clone一份指针给read_sender_mess用
        let mut conn_map_cp = conn_map.clone();

        //读取sender的数据
        read_sender_mess(rec,conn_map_cp);
        info!("TCP-SERVER listening on:{:?}",addr);
        // Register the server with poll we can receive events for it.
        poll.registry()
            .register(&mut server, SERVER, Interest::READABLE)?;
        loop {
            poll.poll(&mut events, None)?;
            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        // Received an event for the TCP server socket.
                        // Accept an connection.
                        let (mut connection, address) = server.accept()?;
                        connection.set_nodelay(true);
                        let token = next(&mut unique_token);
                        info!("Accepted connection from: {}", address);
                        //clone a handler for tcpstream
                        let mut hd = handler.try_clone();
                        hd.on_open(TcpSender{sender:sender.clone(),token:token.0});

                        //缓存handler
                        handler_map.insert(token.0,hd);

                        //为每个tcpstream注册事件
                        poll.registry().register(
                            &mut connection,
                            token,
                            Interest::READABLE.add(Interest::WRITABLE),
                        )?;
                        conn_map.write().unwrap().insert(token.0, connection);
                    }
                    token => {
                        // (maybe) received an event for a TCP connection.
                        let done = if let Some(connection) = conn_map.write().unwrap().get_mut(&token.0) {
                            let hd = handler_map.get_mut(&token.0);
                            match hd {
                                Some(hd)=>{
                                    handle_connection_event(poll.registry(), connection, event,hd)?
                                },
                                None=>{
                                    error!("handler_map has no handler for token:{}",token.0);
                                    false
                                }
                            }
                        } else {
                            // Sporadic events happen.
                            false
                        };
                        if done {
                            conn_map.write().unwrap().remove(&token.0);
                            handler_map.remove(&token.0);
                        }
                    }
                }
            }
        }
    }

    ///Read the data from the sender of the handler
    fn read_sender_mess(rec:Receiver<Data>,connections:Arc<RwLock<HashMap<usize,MioTcpStream>>>){
        let m =  move ||{
            loop {
                let mut result = rec.recv();
                match result {
                    Ok(data)=>{
                        let token = data.token;
                        let bytes = data.bytes;
                        let mut write = connections.write().unwrap();
                        let res:Option<&mut MioTcpStream> = write.get_mut(&token);
                        match res {
                            Some(ts)=>{
                                ts.write(bytes.as_slice());
                                ts.flush();
                            },
                            None=>{
                                error!("connections has no value for token:{}",token);
                            }
                        }
                    },
                    Err(e)=>{
                        error!("{:?}",e.description());
                    }
                }
            }

        };
        std::thread::spawn(m);
    }


    ///add the token
    fn next(current: &mut Token) -> Token {
        let next = current.0;
        current.0 += 1;
        Token(next)
    }

    /// Returns `true` if the connection is done.
    fn handle_connection_event<T: Handler>(
        registry: &Registry,
        connection: &mut MioTcpStream,
        event: &Event,mut handler: &mut T
    ) -> io::Result<bool> {
        // if event.is_writable() {
        //
        //     // We can (maybe) write to the connection.
        //     match connection.write(DATA) {
        //         // We want to write the entire `DATA` buffer in a single go. If we
        //         // write less we'll return a short write error (same as
        //         // `io::Write::write_all` does).
        //         Ok(n) if n < DATA.len() => return Err(io::ErrorKind::WriteZero.into()),
        //         Ok(_) => {
        //             // After we've written something we'll reregister the connection
        //             // to only respond to readable events.
        //             registry.reregister(connection, event.token(), Interest::READABLE)?
        //         }
        //         // Would block "errors" are the OS's way of saying that the
        //         // connection is not actually ready to perform this I/O operation.
        //         Err(ref err) if would_block(err) => {}
        //         // Got interrupted (how rude!), we'll try again.
        //         Err(ref err) if interrupted(err) => {
        //             return handle_connection_event(registry, connection, event,handler)
        //         }
        //         // Other errors we'll consider fatal.
        //         Err(err) => return Err(err),
        //     }
        // }

        // We can (maybe) read from the connection.
        if event.is_readable() {
            loop {
                let mut buf = [0; 256];
                match connection.read(&mut buf) {
                    Ok(0) => {
                        // Reading 0 bytes means the other side has closed the
                        // connection or is done writing, then so are we.
                        connection.shutdown(Shutdown::Both);
                        handler.on_close();
                        return Ok(true);
                    }
                    Ok(n) => {
                        let mut received_data = Vec::new();
                        received_data.extend_from_slice(&buf[..n]);
                        handler.on_message(received_data);
                    },
                    // Would block "errors" are the OS's way of saying that the
                    // connection is not actually ready to perform this I/O operation.
                    Err(ref err) if would_block(err) => {
                        //println!("{:?}",err.description());
                        break;
                    },
                    Err(ref err) if interrupted(err) => {
                        //println!("{:?}",err.description());
                        continue
                    },
                    // Other errors we'll consider fatal.
                    Err(err) => return Err(err),
                }
            }
        }
        Ok(false)
    }

    fn would_block(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::WouldBlock
    }

    fn interrupted(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::Interrupted
    }
}

///TCP client handler, used to extend TCP events
pub trait ClientHandler: Send + Sync {
    ///Called when the connection  open
    fn on_open(&mut self, ts: TcpStream);
    ///called when connection was closed
    fn on_close(&mut self);
    ///called when have mess from server
    fn on_message(&mut self, mess: Vec<u8>);
    ///get address
    fn get_address(&self) -> &str;
    ///start read mess from server
    fn on_read(&mut self, address: String) {
        let mut read = new_tcp_client(address.as_str());
        let  write = read.try_clone().unwrap();
        self.on_open(write);
        let mut read_bytes: [u8; 512] = [0; 512];
        info!("start read from {:?}", address);
        loop {
            //读取从tcp服务端发过来的数据
            let size = read.read(&mut read_bytes);
            if size.is_err() {
                error!("TCP-CLIENT:{:?}", size.unwrap_err());
                self.on_close();
                break;
            }

            let size = size.unwrap();
            if size == 0 {
                info!("tcp客户端断开链接！尝试链接服务器！");
                self.on_close();
                break;
            }
            //如果读取到的字节数大于0则交给handler
            if size > 0 as usize {
                info!(
                    "TCP-CLIENT:data of message from server {:?} ,data size:{}",
                    address, size
                );
                //读取到字节交给handler处理来处理
                let mut v = read_bytes.to_vec();
                v.resize(size, 0);
                self.on_message(v);
            }
        }
    }
}

///new tcp client
pub fn new_tcp_client(address: &str) -> TcpStream {
    let mut ts: Option<std::io::Result<TcpStream>> = None;
    let mut result: Option<TcpStream> = None;
    let dur = Duration::from_secs(2);
    loop {
        ts = Some(connect(address));
        let re = ts.unwrap();
        if re.is_err() {
            error!(
                "连接服务器失败！{:?},{}",
                address,
                re.err().unwrap().to_string()
            );
            //睡2s
            std::thread::sleep(dur);
            continue;
        }
        result = Some(re.unwrap());
        break;
    }

    //设置非阻塞
    let result = result.unwrap();
    //设置参数
    set_tream_param(&result);
    //set_tream_param(&result);
    info!("连接服务器成功！{:?}", address);
    result
}

///set tcp params
fn set_tream_param(ts: &TcpStream) {
    ///No package, direct send
    ts.set_nodelay(true).unwrap();
    //ts.set_read_timeout(Some(Duration::from_millis(50)))
    ///TCP receive buffer size
    ts.set_recv_buffer_size(1024*16 as usize).unwrap();
    ///TCP send buffer size
    ts.set_send_buffer_size(1024*16 as usize).unwrap();
    ///When TCP is off, wait 5s for the data to be processed
    let d = Duration::from_secs(5);
    ts.set_linger(Some(d)).unwrap();
    ///tTcp2 is tested connection every 2 hours
    let d = Duration::from_secs(3600*2);
    ts.set_keepalive(Some(d)).unwrap();
}

///New TCP connection (for client)
fn connect(address: &str) -> std::io::Result<TcpStream> {
    let mut ts = TcpStream::connect(address);
    ts
}
