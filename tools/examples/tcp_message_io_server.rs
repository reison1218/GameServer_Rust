use async_trait::async_trait;
use tools::tcp_message_io::{MessageHandler, TransportWay};

#[derive(Default, Clone)]
pub struct MyServerHandler {
    pub tcp: Option<tools::tcp_message_io::TcpHandler>,
}

#[async_trait]
impl MessageHandler for MyServerHandler {
    async fn try_clone(&self) -> Self {
        self.clone()
    }

    async fn on_open(&mut self, tcp_handler: tools::tcp_message_io::TcpHandler) {
        self.tcp = Some(tcp_handler);
        println!("an new client connected!");
    }

    async fn on_close(&mut self) {
        println!(
            "disconnect with client! address:{:?}",
            self.tcp.as_ref().unwrap().endpoint.addr()
        );
    }

    async fn on_message(&mut self, mess: &[u8]) {
        println!("get mess({:?}) from client!now send back!", mess);
        let str = "hello".to_owned();
        self.tcp.as_mut().unwrap().send(str.as_bytes());
    }
}

pub fn main() {
    tools::tcp_message_io::run(
        TransportWay::Tcp,
        "127.0.0.1:8888",
        MyServerHandler::default(),
    );
}
