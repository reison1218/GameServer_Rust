use super::*;
use net2::TcpStreamExt;
use std::io::Read;
use std::marker::{Send, Sync};
use std::net::Shutdown;
use std::net::TcpStream;
use std::time::Duration;

///The TCP server side handler is used to handle TCP general events, such as connections,
/// closing connections, having data transfers
pub trait Handler: Send + Sync {
    fn try_clone(&self) -> Self;
    ///Triggered when there is a new client connection
    fn on_open(&mut self, sender: TcpSender);

    ///Disconnect triggered when client was closed
    fn on_close(&mut self);

    ///Triggered when there is client data transfer
    fn on_message(&mut self, mess: Vec<u8>);
}

///tcp server sender
#[derive(Clone,Debug)]
pub struct TcpSender {
    pub sender: crossbeam::channel::Sender<Data>,
    pub token: usize,
}

impl TcpSender {
    pub fn write(&mut self, bytes: Vec<u8>){
        let res = self.sender.send(Data {
            bytes,
            token: self.token,
        });
        match res {
            Ok(_)=>{}
            Err(e)=>{
                error!("{:?}",e);
            }
        }
    }

    fn get_token(&self) -> usize {
        self.token
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
///     fn on_open(&mut self, sender: TcpSender) {
///         //do something here what u need
///      }
///     ///Called when the client connection is invalid
///     fn on_close(&mut self) {
///         //do something here what u need
///     }
///     ///Called when has message from client
///     fn on_message(&mut self, mess: Vec<u8>) {
///         //do something here what u need
///     }
/// }
/// ```
///
pub mod tcp_server {
    use super::*;
    use mio::event::Event;
    use mio::net::{TcpListener as MioTcpListener, TcpStream as MioTcpStream};
    use mio::{Events, Interest, Poll, Registry, Token};
    use std::collections::hash_map::HashMap;
    use std::io::{self, Read, Write};
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::sync::{Arc, RwLock};
    use std::error::Error;

    ///事件的唯一标示
    const SERVER: Token = Token(0);

    ///Create the TCP server and start listening on the port
    pub fn new<T: Handler>(addr: &str, handler: T) -> io::Result<()> {
        // Create a poll instance.
        let mut poll = Poll::new()?;
        // Create storage for events.
        let mut events = Events::with_capacity(5120);
        // tcp listenner address
        let address = SocketAddr::from_str(addr).unwrap();
        // Setup the TCP server socket.
        let mut server = MioTcpListener::bind(address)?;

        // Map of `Token` -> `TcpStream`.
        let conn_map = Arc::new(RwLock::new(HashMap::new()));
        //handlermap
        let mut handler_map = HashMap::new();
        // Unique token for each incoming connection.
        let mut unique_token = Token(SERVER.0 + 1);
        // async_channel message ，for receiver all sender of handler's message
        let (sender, rec) = crossbeam::crossbeam_channel::bounded(102400);
        //clone an conn_map to read_sender_mess func
        let conn_map_cp = conn_map.clone();

        //读取sender的数据
        read_sender_mess(rec, conn_map_cp);
        info!("TCP-SERVER listening on:{:?}", addr);
        // Register the server with poll we can receive events for it.
        poll.registry()
            .register(&mut server, SERVER, Interest::READABLE)?;
        loop {
            let res = poll.poll(&mut events, None);
            match res {
                Ok(_)=>{},
                Err(e)=>{
                    warn!("{:?}",e);
                    continue;
                }
            }
            for event in events.iter(){
                match event.token() {
                    SERVER => {
                        // Received an event for the TCP server socket.
                        // Accept an connection.
                        let result: std::io::Result<(MioTcpStream, SocketAddr)> = server.accept();
                        // if is error,print it and continue;
                        if result.is_err() {
                            error!("{:?}", result.err().unwrap());
                            continue;
                        }
                        let (mut connection, client_address) = result.unwrap();
                        connection.set_nodelay(true).unwrap();
                        let token = next(&mut unique_token);
                        //clone a handler for tcpstream
                        let mut hd = handler.try_clone();
                        //trigger the open event
                        hd.on_open(TcpSender {
                            sender: sender.clone(),
                            token: token.0,
                        });

                        //save the handler
                        handler_map.insert(token.0, hd);

                        //register event for every tcpstream
                        let res = poll.registry().register(
                            &mut connection,
                            token,
                            Interest::READABLE.add(Interest::WRITABLE),
                        );
                        if res.is_err(){
                            error!("{:?}",res.err().unwrap());
                            continue;
                        }
                        conn_map.write().unwrap().insert(token.0, connection);
                        info!("Accepted connection from: {}", client_address);
                    }
                    token => {
                        // (maybe) received an event for a TCP connection.
                        let done = if let Some(connection) =
                            conn_map.write().unwrap().get_mut(&token.0)
                        {
                            let hd = handler_map.get_mut(&token.0);
                            match hd {
                                Some(hd) => {
                                    let res = handle_connection_event(poll.registry(), connection, event, hd);
                                    if res.is_err(){
                                        error!("{:?}",res.err().unwrap());
                                        continue;
                                    }
                                    res.unwrap()
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
        rec: crossbeam::channel::Receiver<Data>,
        connections: Arc<RwLock<HashMap<usize, MioTcpStream>>>,
    ) {
        let m = move || {
            loop {
                let result = rec.recv();
                match result {
                    Ok(data) => {
                        let token = data.token;
                        let bytes = data.bytes;
                        let mut write = connections.write().unwrap();
                        let res: Option<&mut MioTcpStream> = write.get_mut(&token);
                        match res {
                            Some(ts) => {
                                //send mess to client
                                let res = ts.write(bytes.as_slice());
                                if let Err(e) = res{
                                    error!("{:?}",e);
                                    continue;
                                }
                                let res = ts.flush();
                                if let Err(e) = res{
                                    error!("{:?}",e);
                                }
                            }
                            None => {
                                warn!("connections has no value for token:{}", token);
                            }
                        }
                    },
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
                let mut buf = [0; 25600];
                match connection.read(&mut buf) {
                    Ok(0) => {
                        // Reading 0 bytes means the other side has closed the
                        // connection or is done writing, then so are we.
                        close_connect(connection,handler,None);
                        return Ok(true);
                    }
                    Ok(n) => {
                        let mut received_data = Vec::new();
                        received_data.extend_from_slice(&buf[..n]);
                        handler.on_message(received_data);
                    }
                    // Would block "errors" are the OS's way of saying that the
                    // connection is  unavailable ready to perform this I/O operation.
                    Err(ref err) if would_block(err) => {
                        let status = err.raw_os_error();
                        let mut res = 0;
                        if status.is_some(){
                            res = status.unwrap();
                        }
                        //系统错误码35代表OSX内核下的socket unactually,错误码11代表linux内核的socket unactually
                        //直接跳出token读取事件，待下次actually再进行读取
                        if res == 35 || res == 11{
                            break;
                        }else{
                            warn!("{:?}",err.to_string());
                        }
                        close_connect(connection,handler,Some(err));
                        return Ok(true);
                        //break;
                    }
                    Err(ref err) if interrupted(err) => {
                        warn!("{:?}",err);
                        continue;
                    },
                    Err(ref err) if other(err) => {
                        warn!("{:?}",err);
                        continue;
                    },
                    // Other errors we'll consider fatal.
                    Err(err) => {
                        warn!("err:{:?}",err);
                        close_connect(connection,handler,Some(&err));
                        //return Err(err)
                        return Ok(true);
                    },
                }
            }
        }
        Ok(false)
    }

    fn close_connect<T:Handler>(connect:&mut MioTcpStream,handler: &mut T,err:Option<&dyn Error>){
        let addr = connect.peer_addr();
        let err_str;
        match err {
            Some(e)=>{err_str = e.to_string()},
            _=>{err_str = "".to_owned()}
        }
        match addr {
            Ok(add) => {
                info!("{:?} client disconnect!so remove client peer:{:?}",err_str, add);
            }
            Err(_) => {
                info!("{:?},then remove client", err_str);
            },
        }
        let res = connect.shutdown(Shutdown::Both);
        if res.is_err(){
            //error!("shutdown TcpStream has error:{:?}",res.err().unwrap().to_string());
        }
        handler.on_close();
    }

    fn would_block(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::WouldBlock
    }

    fn interrupted(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::Interrupted
    }

    fn time_out(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::TimedOut
    }

    fn other(err: &io::Error) -> bool {
        err.kind() == io::ErrorKind::Other
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
        let write = read.try_clone().unwrap();
        self.on_open(write);
        let mut read_bytes: [u8; 51200] = [0; 51200];
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
                //读取到字节交给handler处理来处理
                let mut v = read_bytes.to_vec();
                v.resize(size, 0);
                self.on_message(v);
            }
        }
    }
}

///new tcp client
#[warn(unused_assignments)]
pub fn new_tcp_client(address: &str) -> TcpStream {
    let mut ts: Option<std::io::Result<TcpStream>>;
    let result: Option<TcpStream>;
    let dur = Duration::from_secs(5);
    loop {
        ts = Some(connect(address));
        let res = ts.unwrap();
        if let Err(e) = res{
            error!("连接服务器失败！{:?},{}", address,e.to_string());
            //睡5s
            std::thread::sleep(dur);
            continue;
        }
        result = Some(res.unwrap());
        break;
    }

    let result = result.unwrap();
    //设置参数
    set_tream_param(&result);
    info!("连接服务器成功！{:?}", address);
    result
}

///set tcp params
fn set_tream_param(ts: &TcpStream) {
    //No package, direct send
    ts.set_nodelay(true).unwrap();
    //ts.set_read_timeout(Some(Duration::from_millis(50)))
    //TCP receive buffer size
    ts.set_recv_buffer_size(1024 * 16 as usize).unwrap();
    //TCP send buffer size
    ts.set_send_buffer_size(1024 * 16 as usize).unwrap();
    //When TCP is off, wait 5s for the data to be processed
    let d = Duration::from_secs(5);
    ts.set_linger(Some(d)).unwrap();
    //tTcp2 is tested connection every 2 hours
    let d = Duration::from_secs(3600 * 2);
    ts.set_keepalive(Some(d)).unwrap();
}

///New TCP connection (for client)
fn connect(address: &str) -> std::io::Result<TcpStream> {
    let ts = TcpStream::connect(address);
    ts
}