use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use log::{error, info};
use std::thread::spawn;
use futures::{SinkExt, StreamExt};
use futures::stream::SplitSink;
use tokio_tungstenite::tungstenite::{accept, protocol::Role, Message, WebSocket};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

pub trait MessageHandler: Send + Sync + Clone {
    ///Triggered when has client connected
    fn on_open(&mut self, ws_handler: WsHandler);

    ///Disconnect triggered when client was closed
    fn on_close(&mut self);

    ///Triggered when there is client data transfer
    fn on_message(&mut self, mess: WsMessage);
}

pub struct WsHandler(pub WebSocket<TcpStream>);

impl WsHandler {
    pub fn close(&mut self) {
            let close_frame = CloseFrame {
                code: CloseCode::Normal,  // 正常关闭
                reason: "Normal closure".into(),
            };
            let res = self.0.close(Some(close_frame));
            if let Err(e) = res {
                error!("Error while closing WebSocket connection: {}", e);
            }
    }
    pub fn send(&mut self, mess: WsMessage) {
        let res = self.0.send(mess);
        if let Err(e)=res {
            info!("ws send error: {:?}", e);
        }
    }
}

pub fn build(port: u16, event_callback: impl FnMut(NetEvent) + Send + Clone + 'static) {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let server = TcpListener::bind(addr).unwrap();
    for stream in server.incoming() {
        let stream = stream.unwrap();
        let mut call_back = event_callback.clone();
        spawn(move || {
            info!("new client from {}", stream.peer_addr().unwrap());
            let (mut read_socket, write_socket) = split(stream);

            call_back(NetEvent::Connected(WsHandler(write_socket)));
            loop {
                let msg = read_socket.read();

                match msg {
                    Ok(msg) => {
                        match msg {
                            Message::Ping(ping) => {
                                read_socket.send(Message::from(ping)).unwrap();
                            }
                            Message::Pong(pong) => {
                                read_socket.send(Message::from(pong)).unwrap();
                            }
                            Message::Close(code) => {
                                info!(
                            "client {} disconnect!code:{:?}",
                            read_socket.get_ref().peer_addr().unwrap(),
                            code
                        );
                                call_back(NetEvent::Disconnected);
                                break;
                            }
                            _ => {
                                call_back(NetEvent::Message(msg));
                            }
                        };
                    }
                    Err(err) => {
                        error!("error: {:?}", err);
                        call_back(NetEvent::Disconnected);
                        break;
                    }
                }
            }
        });
    }
}

fn split(tcp_stream: TcpStream) -> (WebSocket<TcpStream>, WebSocket<TcpStream>) {
    //WebSocketConfig::default() 这里websocket配置就用默认配置了
    // WebSocketConfig {
    //     max_send_queue: None,
    //     write_buffer_size: 128 * 1024,
    //     max_write_buffer_size: usize::MAX,
    //     max_message_size: Some(64 << 20),
    //     max_frame_size: Some(16 << 20),
    //     accept_unmasked_frames: false,
    // }
    // 详情请看WebSocketConfig
    let read_socket = accept(tcp_stream.try_clone().unwrap()).unwrap();
    let write_socket =
        WebSocket::from_raw_socket(tcp_stream.try_clone().unwrap(), Role::Server, None);
    (read_socket, write_socket)
}
pub enum NetEvent {
    Connected(WsHandler),

    Message(Message),

    Disconnected,
}

pub type WsMessage = Message;








pub enum ClientNetEvent {
    Connected(WsClientHandler),

    Message(Message),

    Disconnected,
}


pub trait ClientMessageHandler: Send + Sync + Clone {
    ///Triggered when has client connected
    fn on_open(&mut self, ws_handler: WsClientHandler);

    ///Disconnect triggered when client was closed
    fn on_close(&mut self);

    ///Triggered when there is client data transfer
    fn on_message(&mut self, mess: WsMessage);
}

pub struct WsClientHandler(pub Arc<async_lock::Mutex<SplitSink<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>, Message>>>);

impl WsClientHandler{
    pub fn send(&self, mess: WsMessage) {
        smol::block_on(async {
            let res = self.0.lock().await.send(mess).await;
            if let Err(e) = res {
                error!("ws client send error: {:?}", e);
            }
        })
    }


    pub fn close(&mut self) {
        smol::block_on(async {
            let res = self.0.lock().await.close().await;
            if let Err(e) = res {
                error!("ws client close error: {:?}", e);
            }
        });
    }
}


pub fn client_build(url:&str,mut event_callback: impl FnMut(ClientNetEvent) + Send + Clone + 'static){
    let m = async{
        // 建立连接
        let (ws_stream, response) = connect_async(url)
            .await
            .expect("Failed to connect");
        info!("Connected to {}", url);
        // 分离读写部分
        let (write, mut read) = ws_stream.split();

        let mut write_lock = Arc::new(async_lock::Mutex::new(write));
        event_callback(ClientNetEvent::Connected(WsClientHandler(write_lock.clone())));

        loop{
            // 接收响应
            if let Some(msg) = read.next().await {
                match msg {
                    Ok(msg) => {
                        match msg {
                            Message::Ping(ping) => {
                                write_lock.lock().await.send(Message::from(ping)).await.unwrap();
                            }
                            Message::Pong(pong) => {
                                write_lock.lock().await.send(Message::from(pong)).await.unwrap();
                            }
                            Message::Close(code) => {
                                error!("server:{}     disconnect! code:{:?}", url, code);
                                event_callback(ClientNetEvent::Disconnected);
                                break;
                            },
                            _=>{
                                event_callback(ClientNetEvent::Message(msg));
                            }
                        }
                    }
                    Err(e) => {
                        error!("read error:{}  server:{}      disconnect!", e,url);
                        event_callback(ClientNetEvent::Disconnected);
                        break;
                    },
                }
            }
        }
    };
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()  // 启用IO和定时器
        .build()
        .unwrap();
    rt.block_on(m);
}