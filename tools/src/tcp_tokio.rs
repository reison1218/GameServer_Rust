use log::info;
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

pub trait MessageHandler: Send + Sync + Clone {
    ///Triggered when has client connected
    fn on_open(&mut self, tcp_handler: TcpHandler);

    ///Disconnect triggered when client was closed
    fn on_close(&mut self);

    ///Triggered when there is client data transfer
    fn on_message(&mut self, mess: &[u8]);
}

pub struct TcpHandler(pub tokio::net::tcp::OwnedWriteHalf);

impl TcpHandler {
    pub fn send(&mut self, mess: &[u8]) {
        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let res = self.0.write_all(mess).await;
                if let Err(e) = res {
                    log::error!("{:?}", e);
                }

                let res = self.0.flush().await;
                if let Err(e) = res {
                    log::error!("{:?}", e);
                }
            })
        });
    }

    pub fn close(&mut self) {
        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let res = self.0.shutdown().await;
                if let Err(e) = res {
                    log::error!("{:?}", e);
                }
            })
        });
    }
}

pub struct Builder {
    tokio_run_time: tokio::runtime::Builder,
}

impl Builder {
    ///创建一个构造器
    pub fn new() -> Self {
        let run_time = tokio::runtime::Builder::new_multi_thread();
        Builder {
            tokio_run_time: run_time,
        }
        .enable_all()
    }

    //是否开启io和time驱动
    pub fn enable_all(mut self) -> Self {
        self.tokio_run_time.enable_all();
        self
    }

    ///设置最大阻塞线程数量，空闲会销毁，可以配合thread_keep_alive使用。
    pub fn max_blocking_threads(mut self, val: usize) -> Self {
        self.tokio_run_time.max_blocking_threads(val);
        self
    }

    ///设置最大阻塞线程池数量，默认每个线程10s。
    pub fn thread_keep_alive(mut self, dur: Duration) -> Self {
        self.tokio_run_time.thread_keep_alive(dur);
        self
    }

    ///配置I/O驱动程序并最大事件数，参考值1024
    pub fn max_io_events_per_tick(mut self, capacity: usize) -> Self {
        self.tokio_run_time.max_io_events_per_tick(capacity);
        self
    }

    ///配置tokio运行时线程名称
    pub fn thread_name(mut self, name: &str) -> Self {
        self.tokio_run_time.thread_name(name);
        self
    }

    ///配置worker threads 栈大小，默认2M，以后可能会变
    pub fn thread_stack_size(mut self, val: usize) -> Self {
        self.tokio_run_time.thread_stack_size(val);
        self
    }

    ///设置worker threads线程数，默认会配置为系统cpu核心数
    pub fn worker_threads(mut self, val: usize) -> Self {
        self.tokio_run_time.worker_threads(val);
        self
    }

    ///创建网络底层服务并监听
    pub fn build(
        &mut self,
        port: u16,
        event_callback: impl FnMut(NetEvent) + Send + Clone + 'static,
    ) {
        let run_time = self.tokio_run_time.build().unwrap();

        run_time.block_on(async {
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
            let res = TcpListener::bind(addr).await;
            if let Err(e) = res {
                log::error!("{:?}", e);
                return;
            }
            let listener = res.unwrap();

            info!("tcp server start,listen port {}", port);

            loop {
                let res = listener.accept().await;
                if let Err(e) = res {
                    log::error!("{:?}", e);
                    return;
                }
                let (socket, _) = res.unwrap();
                let res = socket.set_linger(Some(Duration::from_millis(5000)));
                if let Err(e) = res {
                    log::error!("{:?}", e);
                    continue;
                }
                let res = socket.set_nodelay(false);
                if let Err(e) = res {
                    log::error!("{:?}", e);
                    continue;
                }
                let res = socket.set_ttl(254);
                if let Err(e) = res {
                    log::error!("{:?}", e);
                    continue;
                }

                let (mut read_stream, write_stream) = socket.into_split();

                let mut call_back = event_callback.clone();

                run_time.spawn(async move {
                    call_back(NetEvent::Connected(TcpHandler(write_stream)));
                    let mut buf = [0; u16::MAX as usize];
                    loop {
                        match read_stream.read(&mut buf).await {
                            Ok(n) => {
                                // socket closed
                                if n == 0 {
                                    call_back(NetEvent::Disconnected);
                                    break;
                                }

                                let mut received_data = Vec::new();
                                received_data.extend_from_slice(&buf[..n]);

                                //call on_message
                                call_back(NetEvent::Message(received_data));
                            }
                            Err(e) => {
                                log::error!("failed to read from socket; err = {:?}", e);
                                println!("server failed to read from socket; err = {:?}", e);
                                break;
                            }
                        }
                    }
                });
            }
        });
    }
}


pub fn connect(
    ip: &str,
    port: u16,
    mut event_callback: impl FnMut(NetEvent) + Send + Clone + 'static,
) {
    let ip = ip.to_owned();
    let m = move ||{
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on( async move{
            let (mut read_stream,write_stream) = tokio::net::TcpStream::connect(format!("{}:{}", ip, port)).await.unwrap().into_split();
            event_callback(NetEvent::Connected(TcpHandler(write_stream)));
            let mut buf = [0; u16::MAX as usize];
            loop{
                match read_stream.read(&mut buf).await {
                    Ok(n) => {
                        // socket closed
                        if n == 0 {
                            event_callback(NetEvent::Disconnected);
                            break;
                        }

                        let mut received_data = Vec::new();
                        received_data.extend_from_slice(&buf[..n]);

                        //call on_message
                        event_callback(NetEvent::Message(received_data));
                    }
                    Err(e) => {
                        log::error!("failed to read from socket; err = {:?}", e);
                        break;
                    }
                }
            }
        });
    };
    std::thread::spawn(m);
}


pub enum NetEvent {
    Connected(TcpHandler),

    Message(Vec<u8>),

    Disconnected,
}
