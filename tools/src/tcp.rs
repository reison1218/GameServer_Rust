use super::*;
use async_trait::async_trait;
use crossbeam::channel::{Receiver, Sender};
use net2::TcpStreamExt;
use std::io;
use std::io::{Read, Write};
use std::marker::{Send, Sync};
use std::net::Shutdown;
use std::net::TcpStream;
use std::time::Duration;

///The TCP server side handler is used to handle TCP general events, such as connections,
/// closing connections, having data transfers
#[async_trait]
pub trait Handler: Send + Sync {
    ///try to clone self
    async fn try_clone(&self) -> Self;

    ///Triggered when there is a new client connection
    async fn on_open(&mut self, sender: TcpSender);

    ///Disconnect triggered when client was closed
    async fn on_close(&mut self);

    ///Triggered when there is client data transfer
    ///return the res of verify,if true,that is ok,false is verify fail!
    async fn on_message(&mut self, mess: Vec<u8>) -> bool;
}

///tcp server sender
#[derive(Clone, Debug)]
pub struct TcpSender {
    pub sender: Sender<Data>,
    pub token: usize,
}

impl TcpSender {
    pub fn send(&mut self, bytes: Vec<u8>) {
        let res = self.sender.send(Data {
            bytes,
            token: self.token,
        });
        match res {
            Ok(_) => {}
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
}

/// for TCP hanler end channel to transfer data
#[derive(Clone)]
pub struct Data {
    pub token: usize,
    pub bytes: Vec<u8>,
}

unsafe impl Send for Data {}
unsafe impl Sync for Data {}

//系统错误码35:代表OSX内核下的socket unactually
//const MAC_OS_SOCKET_UNACTUALLY_ERROR_CODE: i32 = 35;

//错误码11代表linux内核的socket unactually
//const LINUX_OS_SOCKET_UNACTUALLY_ERROR_CODE: i32 = 11;

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
///     ///Called when there is a new client connection
///     fn on_open(&mut self, sender: TcpSender){
///         //do something here what u need
///      }
///     ///Called when the client connection is invalid
///     fn on_close(&mut self) {
///         //do something here what u need
///     }
///     ///Called when has message from client
///     fn on_message(&mut self, mess: Vec<u8>) -> bool {
///         //do something here what u need
///     }
/// }
/// ```
///
pub mod tcp_server {

    use super::*;
    use async_std::task::block_on;
    use mio::event::Event;
    use mio::net::{TcpListener as MioTcpListener, TcpStream as MioTcpStream};
    use mio::{Events, Interest, Poll, Registry, Token};
    use std::collections::hash_map::HashMap;
    use std::error::Error;
    use std::io::{self, Read, Write};
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::{Arc, RwLock};

    ///事件的唯一标示
    const SERVER: Token = Token(0);

    ///Create the TCP server and start listening on the port
    pub async fn new(addr: String, handler: impl Handler) -> io::Result<()> {
        // Create a poll instance.
        let mut poll = Poll::new()?;
        // Create storage for events.
        let mut events = Events::with_capacity(5120);
        // tcp listenner address
        let address = SocketAddr::from_str(addr.as_str()).unwrap();
        // Setup the TCP server socket.
        let mut server = MioTcpListener::bind(address)?;
        // Map of `Token` -> `TcpStream`.
        let conn_map = Arc::new(RwLock::new(HashMap::new()));
        //handlermap
        let mut handler_map = HashMap::new();
        // Unique token for each incoming connection.
        let mut unique_token = Token(SERVER.0 + 1);
        // async_channel message ，for receiver all sender of handler's message
        let (sender, rec) = crossbeam::channel::bounded(102400);
        //clone an conn_map to read_sender_mess func
        let conn_map_cp = conn_map.clone();

        //read data from sender
        read_sender_mess(rec, conn_map_cp);
        info!("TCP-SERVER listening on:{:?}", addr);
        info!("server start success!");
        // Register the server with poll we can receive events for it.
        poll.registry()
            .register(&mut server, SERVER, Interest::READABLE)?;
        loop {
            let res = poll.poll(&mut events, None);
            if let Err(e) = res {
                warn!("{:?}", e);
                continue;
            }
            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        // Received an event for the TCP server socket.
                        // Accept an connection.
                        let result: std::io::Result<(MioTcpStream, SocketAddr)> = server.accept();
                        // if is error,print it and continue;
                        if let Err(e) = result {
                            error!("{:?}", e);
                            continue;
                        }

                        let (mut connection, client_address) = result.unwrap();
                        let res = connection.set_nodelay(true);
                        if let Err(e) = res {
                            error!("{:?}", e);
                            continue;
                        }
                        let token = next(&mut unique_token);

                        //register event for every tcpstream
                        let res = poll.registry().register(
                            &mut connection,
                            token,
                            Interest::READABLE.add(Interest::WRITABLE),
                        );
                        if let Err(e) = res {
                            error!("{:?}", e);
                            continue;
                        }
                        conn_map.write().unwrap().insert(token.0, connection);
                        info!("Accepted connection from: {}", client_address);

                        //clone a handler for tcpstream
                        let mut hd = handler.try_clone().await;

                        //trigger the open event
                        let res = hd.on_open(TcpSender {
                            sender: sender.clone(),
                            token: token.0,
                        });
                        block_on(res);

                        //save the handler
                        handler_map.insert(token.0, hd);
                    }
                    token => {
                        // (maybe) received an event for a TCP connection.
                        let done =
                            if let Some(connection) = conn_map.write().unwrap().get_mut(&token.0) {
                                let hd = handler_map.get_mut(&token.0);
                                match hd {
                                    Some(hd) => {
                                        let res = handle_connection_event(
                                            poll.registry(),
                                            connection,
                                            event,
                                            hd,
                                        );

                                        match res {
                                            Ok(res) => res,
                                            Err(err) => {
                                                error!("{:?}", err);
                                                continue;
                                            }
                                        }
                                    }
                                    None => {
                                        error!("handler_map has no handler for token:{}", token.0);
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
    fn read_sender_mess(
        rec: Receiver<Data>,
        connections: Arc<RwLock<HashMap<usize, MioTcpStream>>>,
    ) {
        let m = move || {
            loop {
                let result = rec.recv();
                match result {
                    Ok(data) => {
                        let token = data.token;
                        let bytes = data.bytes;
                        let write = connections.write();
                        if let Err(e) = write {
                            error!("{:?}", e);
                            continue;
                        }
                        let mut write = write.unwrap();

                        let res: Option<&mut MioTcpStream> = write.get_mut(&token);
                        match res {
                            Some(ts) => {
                                //send mess to client
                                let res = ts.write(bytes.as_slice());
                                match res {
                                    Ok(_) => {}
                                    Err(ref err)
                                        if reset(err)
                                            | connection_refused(err)
                                            | aborted(err)
                                            | not_connected(err)
                                            | broken_pipe(err) =>
                                    {
                                        error!("{:?}", err);
                                        let res = ts.shutdown(Shutdown::Both);
                                        if let Err(e) = res {
                                            warn!("{:?}", e);
                                        }
                                        break;
                                    }
                                    Err(e) => {
                                        error!("{:?}", e);
                                        continue;
                                    }
                                }

                                let res = ts.flush();
                                if let Err(e) = res {
                                    error!("{:?}", e);
                                }
                            }
                            None => {
                                warn!("connections has no value for token:{}", token);
                            }
                        }
                    }
                    Err(e) => {
                        error!("{:?}", e);
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
        _: &Registry,
        connection: &mut MioTcpStream,
        event: &Event,
        handler: &mut T,
    ) -> io::Result<bool> {
        // We can (maybe) read from the connection.
        if event.is_readable() {
            let mut buf = [0; 51200];
            loop {
                let read_res = connection.read(&mut buf);
                match read_res {
                    Ok(0) => {
                        // Reading 0 bytes means the other side has closed the
                        // connection or is done writing, then so are we.
                        close_connect(connection, handler, None);
                        return Ok(true);
                    }
                    Ok(n) => {
                        let mut received_data = Vec::new();
                        received_data.extend_from_slice(&buf[..n]);
                        let res = block_on(handler.on_message(received_data));
                        if !res {
                            warn!("data verify fail!kick this tcp client!");
                            close_connect(connection, handler, None);
                            return Ok(true);
                        }
                    }
                    // Would block "errors" are the OS's way of saying that the
                    // connection is  unavailable ready to perform this I/O operation.
                    Err(ref err) if would_block(err) => {
                        break;
                    }
                    Err(ref err) if interrupted(err) => {
                        warn!("{:?}", err);
                        continue;
                    }
                    Err(ref err) if time_out(err) => {
                        //warn!("{:?}",err);
                        continue;
                    }

                    Err(ref err)
                        if reset(err)
                            | connection_refused(err)
                            | aborted(err)
                            | not_connected(err)
                            | broken_pipe(err) =>
                    {
                        close_connect(connection, handler, Some(err));
                        return Ok(true);
                    }

                    Err(ref err) if other(err) => {
                        warn!("{:?}", err);
                        continue;
                    }
                    // Other errors we'll consider fatal.
                    Err(err) => {
                        close_connect(connection, handler, Some(&err));
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }

    ///socket close event
    fn close_connect<T: Handler>(
        connect: &mut MioTcpStream,
        handler: &mut T,
        err: Option<&dyn Error>,
    ) {
        let addr = connect.peer_addr();
        let err_str;
        match err {
            Some(e) => err_str = e.to_string(),
            _ => err_str = "".to_owned(),
        }
        match addr {
            Ok(add) => {
                info!("client disconnect!so remove client peer:{:?}", add);
            }
            Err(_) => {
                warn!("{:?},then remove client", err_str);
            }
        }
        let _ = connect.shutdown(Shutdown::Both);
        block_on(handler.on_close());
    }
}

pub fn would_block(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::WouldBlock
}

pub fn interrupted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Interrupted
}

pub fn time_out(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::TimedOut
}

pub fn aborted(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::ConnectionAborted
}

pub fn not_connected(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::NotConnected
}

pub fn connection_refused(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::ConnectionRefused
}

pub fn reset(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::ConnectionReset
}

pub fn broken_pipe(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::BrokenPipe
}

pub fn other(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::Other
}

///TCP client handler, used to extend TCP events
#[async_trait]
pub trait ClientHandler: Send + Sync {
    ///Called when the connection  open
    async fn on_open(&mut self, sender: Sender<Vec<u8>>);
    ///called when connection was closed
    async fn on_close(&mut self);
    ///called when have mess from server
    async fn on_message(&mut self, mess: Vec<u8>);
    ///start read mess from server
    async fn on_read(&mut self, address: String) {
        let read = new_tcp_client(address.as_str());
        if let Err(e) = read {
            error!("{:?}", e);
            return;
        }
        let mut read = read.unwrap();
        let write = read.try_clone();
        if let Err(e) = write {
            error!("{:?}", e);
            return;
        }
        let write = write.unwrap();
        let (sender, rec) = crossbeam::channel::bounded(102400);
        //start reading the sender message
        read_sender_mess_client(rec, write);
        //trigger socket open event
        self.on_open(sender.clone()).await;
        //u8 array,for read data from socket client
        let mut read_bytes: [u8; 51200] = [0; 51200];
        info!("start read from {:?}", address);
        loop {
            //start read
            let size = read.read(&mut read_bytes);
            match size {
                Ok(size) => {
                    if size == 0 {
                        info!("tcp客户端断开链接！尝试链接服务器！");
                        self.on_close().await;
                        break;
                    }
                    //如果读取到的字节数大于0则交给handler
                    if size > 0 {
                        //读取到字节交给handler处理来处理
                        let mut v = Vec::new();
                        v.extend_from_slice(&read_bytes[..size]);
                        self.on_message(v).await;
                    }
                }
                // Would block "errors" are the OS's way of saying that the
                // connection is not actually ready to perform this I/O operation.
                Err(ref err) if would_block(err) => {
                    continue;
                }
                Err(ref err) if interrupted(err) => {
                    warn!("{:?}", err);
                    continue;
                }
                Err(ref err) if time_out(err) => {
                    //warn!("{:?}",err);
                    continue;
                }
                Err(ref err)
                    if reset(err)
                        | connection_refused(err)
                        | aborted(err)
                        | not_connected(err)
                        | broken_pipe(err) =>
                {
                    self.on_close().await;
                    break;
                }
                Err(ref err) if other(err) => {
                    warn!("{:?}", err);
                    continue;
                }
                // Other errors we'll consider fatal.
                Err(err) => {
                    error!("TCP-CLIENT:{:?}", err);
                    self.on_close().await;
                    break;
                }
            }
        }
    }
}

///Read the data from the sender of the handler
fn read_sender_mess_client(rec: Receiver<Vec<u8>>, tcp_stream: std::net::TcpStream) {
    let mut tcp_stream = tcp_stream;
    let m = move || loop {
        let result = rec.recv();
        match result {
            Ok(data) => {
                let bytes = data;
                let write = tcp_stream.write(&bytes[..]);

                match write {
                    Ok(_) => {}
                    Err(ref err)
                        if reset(err)
                            | connection_refused(err)
                            | aborted(err)
                            | not_connected(err)
                            | broken_pipe(err) =>
                    {
                        warn!("{:?}", err);
                        let _ = tcp_stream.shutdown(Shutdown::Both);
                        break;
                    }
                    Err(ref err) => {
                        error!("{:?}", err);
                        continue;
                    }
                };
                let res = tcp_stream.flush();
                if let Err(e) = res {
                    error!("{:?}", e);
                }
            }
            Err(e) => {
                error!("{:?}", e);
                break;
            }
        }
    };

    std::thread::spawn(m);
}

///new tcp client
#[warn(unused_assignments)]
fn new_tcp_client(address: &str) -> anyhow::Result<TcpStream> {
    let mut ts: Option<std::io::Result<TcpStream>>;
    let result: Option<TcpStream>;
    let dur = Duration::from_secs(5);
    loop {
        ts = Some(connect(address));
        let res = ts.unwrap();
        if let Err(e) = res {
            error!("连接服务器失败！{:?},{}", address, e.to_string());
            //睡5s
            std::thread::sleep(dur);
            continue;
        }
        result = Some(res.unwrap());
        break;
    }

    let result = result.unwrap();
    //设置参数
    set_tream_param(&result)?;
    info!("连接服务器成功！{:?}", address);
    Ok(result)
}

///set tcp params
fn set_tream_param(ts: &TcpStream) -> anyhow::Result<()> {
    //No package, direct send
    ts.set_nodelay(true)?;
    //ts.set_read_timeout(Some(Duration::from_millis(50)))
    //TCP receive buffer size
    ts.set_recv_buffer_size(1024 * 16 as usize)?;
    //TCP send buffer size
    ts.set_send_buffer_size(1024 * 16 as usize)?;
    //When TCP is off, wait 5s for the data to be processed
    let d = Duration::from_secs(5);
    ts.set_linger(Some(d))?;
    //tTcp2 is tested connection every 2 hours
    let d = Duration::from_secs(3600 * 2);
    ts.set_keepalive(Some(d))?;
    Ok(())
}

///New TCP connection (for client)
fn connect(address: &str) -> std::io::Result<TcpStream> {
    let ts = TcpStream::connect(address);
    ts
}
