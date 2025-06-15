use std::thread::spawn;
use tools::ws::{self, ClientMessageHandler, ClientNetEvent, MessageHandler, NetEvent, WsClientHandler, WsHandler, WsMessage};

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
        println!("rec mess from client {:?}", mess);
        self.handler
            .as_mut()
            .unwrap().send(WsMessage::text("hello client".to_string()));
        self.handler.as_mut().unwrap().close();
    }
}











#[derive(Default)]
struct WsClient {
    pub handler: Option<WsClientHandler>,
}

impl Clone for WsClient {
    fn clone(&self) -> Self {
        Self { handler: None }
    }
}

impl ClientMessageHandler for WsClient {
    fn on_open(&mut self, ws_handler: WsClientHandler) {
        self.handler = Some(ws_handler);
        self.handler.as_mut().unwrap().send(WsMessage::text("hello server".to_string()));
    }

    fn on_close(&mut self) {
        println!(
            "tcp server close!",
        );
    }

    fn on_message(&mut self, mess: WsMessage) {
        println!("rec mess from server {:?}", mess);
        self.handler
            .as_mut()
            .unwrap()
            .send(WsMessage::text("hello server".to_string()));
    }
}
fn client() {
    let mut client = WsClient { handler: None };
    tools::ws::client_build("ws://localhost:1090/socket",move |event| match event {
        ClientNetEvent::Connected(tcp_handler) => {
            client.on_open(tcp_handler);
        }
        ClientNetEvent::Message(data) => {
            client.on_message(data);
        }
        ClientNetEvent::Disconnected => {
            client.on_close();
        }
    });
}
