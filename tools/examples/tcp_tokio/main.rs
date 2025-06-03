use std::thread::spawn;
use std::time::Duration;

use message_io::network::Transport;
use message_io::node::{self, NodeEvent};

use tools::tcp_tokio;
use tools::tcp_tokio::{MessageHandler, NetEvent, TcpHandler};

pub fn main() {
    spawn(move || {
        server_fn();
    });
    client_fn();

    std::thread::park();
}

pub fn server_fn() {
    let mut client = Client { handler: None };
    spawn(move || {
        tcp_tokio::Builder::new().build(1080, move |event| match event {
            NetEvent::Connected(tcp_handler) => {
                client.on_open(tcp_handler);
            }
            NetEvent::Message(data) => {
                client.on_message(&data);
            }
            NetEvent::Disconnected => {
                client.on_close();
            }
        });
    });
}

#[derive(Default)]
struct Client {
    pub handler: Option<TcpHandler>,
}

#[derive(Default)]
struct ClientStream {
    pub handler: Option<TcpHandler>,
}


impl Clone for ClientStream {
    fn clone(&self) -> Self {
        Self { handler: None }
    }
}


impl Clone for Client {
    fn clone(&self) -> Self {
        Self { handler: None }
    }
}

impl MessageHandler for Client {
    fn on_open(&mut self, tcp_handler: TcpHandler) {
        self.handler = Some(tcp_handler);
        println!(
            "new tcp client connect!{}",
            self.handler.as_ref().unwrap().0.peer_addr().unwrap()
        );
    }

    fn on_close(&mut self) {
        println!(
            "tcp client close!{}",
            self.handler.as_ref().unwrap().0.peer_addr().unwrap()
        );
    }

    fn on_message(&mut self, mess: &[u8]) {
        println!("rec mess from client{:?}", mess);
        self.handler.as_mut().unwrap().send(mess);
    }
}

pub fn client_fn(){
    let mut cs = ClientStream::default();
    tcp_tokio::connect("127.0.0.1",1080,move |event| match event {
        NetEvent::Connected( tcp_handler) => {
            cs.handler = Some(tcp_handler);
            println!("connected to server!");
        }
        NetEvent::Message(data) => {
            println!("message from server: {:?}", data);
        }
        NetEvent::Disconnected => {
            println!("close by server");
        }
    });
}