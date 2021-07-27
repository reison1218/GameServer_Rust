use async_trait::async_trait;
use tools::tcp_message_io::{MessageHandler, TransportWay};

#[derive(Default, Clone)]
pub struct MyServerHandler {
    tcp: Option<tools::tcp_message_io::TcpHandler>,
}

#[async_trait]
impl MessageHandler for MyServerHandler {
    async fn try_clone(&self) -> Self {
        self.clone()
    }

    async fn on_open(&mut self, tcp_handler: tools::tcp_message_io::TcpHandler) {
        self.tcp = Some(tcp_handler);
        println!("connect to server success!");
    }

    async fn on_close(&mut self) {
        println!(
            "disconnect with server! address:{:?}",
            self.tcp.as_ref().unwrap().endpoint.addr()
        );
    }

    async fn on_message(&mut self, mess: &[u8]) {
        println!("get mess({:?}) from server!now send back!", mess);
        self.tcp.as_mut().unwrap().send(b"hello i am client!");
    }
}

pub fn main() {
    let a = async {
        let mut mh = MyServerHandler::default();
        mh.connect(TransportWay::Tcp, "127.0.0.1:8888").await;
    };
    async_std::task::block_on(a);
}
