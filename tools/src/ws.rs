use std::sync::Arc;
use log::{error, info};
use tokio::net::{TcpListener,TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, connect_async, MaybeTlsStream, WebSocketStream};
use futures_util::{StreamExt, SinkExt};
use futures_util::stream::SplitSink;

pub trait MessageHandler: Send + Sync + Clone {
    ///Triggered when has client connected
    fn on_open(&mut self, ws_handler: WsHandler);

    ///Disconnect triggered when client was closed
    fn on_close(&mut self);

    ///Triggered when there is client data transfer
    fn on_message(&mut self, mess: WsMessage);
}

pub struct WsHandler(pub Arc<async_lock::Mutex<SplitSink<WebSocketStream<TcpStream>,Message>>>);

impl WsHandler {
    pub fn close(&mut self) {
        let res = smol::block_on(async { self.0.lock().await.close().await });
        if let Err(e) = res {
            error!("Error while closing WebSocket connection: {}", e);
        }
    }
    pub fn send(&mut self, mess: WsMessage) {
        let res = smol::block_on(async { self.0.lock().await.send(mess).await });
        if let Err(e)=res {
            info!("ws send error: {:?}", e);
        }
    }
}

pub async fn build(port: u16, event_callback: impl FnMut(NetEvent) + Send + Clone + 'static) {
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let server = TcpListener::bind(addr).await.unwrap();

    while let Ok((stream, _)) = server.accept().await {
        let mut call_back = event_callback.clone();
        tokio::spawn(async move {
            let client_add = stream.peer_addr().unwrap();
            info!("new client from {}", client_add);

            let ws_stream = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    error!("Error during WebSocket handshake: {}", e);
                    return;
                }
            };

            let (write_socket, mut read_socket) = ws_stream.split();

            let write_lock = Arc::new(async_lock::Mutex::new(write_socket));

            call_back(NetEvent::Connected(WsHandler(write_lock.clone())));

            while let Some(msg) = read_socket.next().await{
                match msg {
                    Ok(msg) => {
                        match msg {
                            Message::Ping(ping) => {
                                write_lock.lock().await.send(WsMessage::Ping(ping)).await.unwrap();
                            }
                            Message::Pong(pong) => {
                                write_lock.lock().await.send(WsMessage::Pong(pong)).await.unwrap();
                            }
                            Message::Close(code) => {
                                info!("client {} disconnect!code:{:?}",client_add,code);
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

pub struct WsClientHandler(pub Arc<async_lock::Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>);

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

    let url = url.to_owned();
    let m = async move {
        // 建立连接
        let (ws_stream, _) = connect_async(url.clone())
            .await
            .expect("Failed to connect");
        info!("Connected to {}", url);
        // 分离读写部分
        let (write, mut read) = ws_stream.split();

        let write_lock = Arc::new(async_lock::Mutex::new(write));
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
    tokio::spawn(m);
}