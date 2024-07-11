use std::thread::spawn;

use tools::ws::{self, MessageHandler, NetEvent, WsHandler, WsMessage};
use tungstenite::connect;

pub fn main() {
    spawn(move || {
        server();
    });
    spawn(move || {
        client();
    });
    std::thread::park();
}

fn server() {
    let mut client = Client { handler: None };
    ws::build(1090, move |event| match event {
        NetEvent::Connected(tcp_handler) => {
            client.on_open(tcp_handler);
        }
        NetEvent::Message(data) => {
            client.on_message(data);
        }
        NetEvent::Disconnected => {
            client.on_close();
        }
    });
}

#[derive(Default)]
struct Client {
    pub handler: Option<WsHandler>,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self { handler: None }
    }
}

impl MessageHandler for Client {
    fn on_open(&mut self, ws_handler: WsHandler) {
        self.handler = Some(ws_handler);
        println!(
            "new ws client connect!{}",
            self.handler
                .as_ref()
                .unwrap()
                .0
                .get_ref()
                .peer_addr()
                .unwrap()
        );
    }

    fn on_close(&mut self) {
        println!(
            "tcp client close!{}",
            self.handler
                .as_ref()
                .unwrap()
                .0
                .get_ref()
                .peer_addr()
                .unwrap()
        );
    }

    fn on_message(&mut self, mess: WsMessage) {
        println!("rec mess from client{:?}", mess);
        self.handler
            .as_mut()
            .unwrap()
            .0
            .send(WsMessage::text("hello".to_string()))
            .unwrap();
    }
}

fn client() {
    let (mut socket, response) = connect("ws://localhost:1090/socket").expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());
    println!("Response contains the following headers:");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    socket
        .send(WsMessage::Text("Hello WebSocket".into()))
        .unwrap();
    loop {
        let msg = socket.read().expect("Error reading message");
        println!("from server message: {:?}", msg);
    }
}
