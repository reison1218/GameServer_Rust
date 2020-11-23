use async_std::sync::RwLock;
use async_trait::async_trait;
use futures::executor::block_on;
use log::error;
use std::sync::Arc;
use tools::tcp::TcpSender;
use tools::tcp::{tcp_server, Handler};

///u can put any data at here
#[derive(Default)]
struct MyData {
    pub sender: Option<TcpSender>,
}

impl MyData {
    pub fn set_sender(&mut self, sender: TcpSender) {
        self.sender = Some(sender);
    }

    pub fn write_2_client(&mut self, data: Vec<u8>) {
        self.sender.as_mut().unwrap().write(data);
    }
}

///this is handler for handler mess from tcp client,every tcp client has their single ServerHandler
///just need impl tools::tcp::Handler for it,then it could be handler mess from client.
#[derive(Default, Clone)]
struct ServerHandler {
    data: Arc<RwLock<MyData>>,
}

#[async_trait]
impl Handler for ServerHandler {
    async fn try_clone(&self) -> Self {
        self.clone()
    }

    async fn on_open(&mut self, sender: TcpSender) {
        println!("has new tcp client coming!");
        self.data.write().await.set_sender(sender);
    }

    async fn on_close(&mut self) {
        println!("oh,tcp client was disconnect");
    }

    async fn on_message(&mut self, mess: Vec<u8>) {
        //todo u can do someting at here
        handler_mess(self.data.clone(), mess).await;
    }
}

async fn handler_mess(data: Arc<RwLock<MyData>>, mess: Vec<u8>) {
    //todo and then,write back to client.like this:
    println!("from client,size:{}", mess.len());
    let str = String::from("hello,client,i am server!");
    let mut write_lock = data.write().await;
    write_lock.write_2_client(str.as_bytes().to_vec());
}

fn main() {
    let address = "127.0.0.1:8080";
    let res = tcp_server::new(address, ServerHandler::default());
    let res = block_on(res);
    if let Err(e) = res {
        error!("{:?}", e);
    }
}
