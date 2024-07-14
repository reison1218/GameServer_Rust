use std::net::{TcpListener, TcpStream};

use log::info;
use std::thread::spawn;
use tungstenite::{accept, protocol::Role, Message, WebSocket};

pub trait MessageHandler: Send + Sync + Clone {
    ///Triggered when has client connected
    fn on_open(&mut self, ws_handler: WsHandler);

    ///Disconnect triggered when client was closed
    fn on_close(&mut self);

    ///Triggered when there is client data transfer
    fn on_message(&mut self, mess: WsMessage);
}

pub struct WsHandler(pub WebSocket<TcpStream>);

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
                let msg = read_socket.read().unwrap();

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
