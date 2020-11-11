use async_std::sync::RwLock;
use async_trait::async_trait;
use log::error;
use std::net::TcpStream;
use std::sync::Arc;
use tools::tcp::ClientHandler;

///u can put any data at here
#[derive(Default)]
pub struct MyData {}

#[derive(Default)]
pub struct TcpClientHandler {
    ts: Option<TcpStream>,
    cp: MyData,
}

#[async_trait]
impl ClientHandler for TcpClientHandler {
    async fn on_open(&mut self, ts: TcpStream) {
        //do something at here
        self.ts = Some(ts);
        println!("connect to tcp server success!");
    }

    async fn on_close(&mut self) {
        //todo u can do something here
    }

    async fn on_message(&mut self, mess: Vec<u8>) {
        async_std::task::spawn(handler_mess(mess));
    }
}

///todo u can do something here
async fn handler_mess(mess: Vec<u8>) {
    //todo do something
}

fn main() {
    let address = String::from("127.0.0.1:8080");
    let mut tc = TcpClientHandler::default();
    async_std::task::block_on(tc.on_read(address));
}
