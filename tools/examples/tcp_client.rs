use async_trait::async_trait;
use crossbeam::channel::Sender;
use tools::tcp::ClientHandler;

///u can put any data at here
#[derive(Default)]
pub struct MyData {}

#[derive(Default)]
pub struct TcpClientHandler {
    ts: Option<Sender<Vec<u8>>>,
}

impl TcpClientHandler {
    ///todo u can do something here
    async fn handler_mess(&mut self, mess: Vec<u8>) {
        //todo do something
        println!("read mess from tcp server!size:{}", mess.len());
        let res = self
            .ts
            .as_mut()
            .unwrap()
            .send(b"hello,client,i am server!".to_vec());
        if let Err(e) = res {
            println!("{:?}", e);
        }
    }
}

#[async_trait]
impl ClientHandler for TcpClientHandler {
    async fn on_open(&mut self, sender: Sender<Vec<u8>>) {
        //do something at here
        self.ts = Some(sender);
        println!("connect to tcp server success!");
    }

    async fn on_close(&mut self) {
        // u can do something here when the tcp was disconnect
        println!("disconnect with tcp server!");
    }

    async fn on_message(&mut self, mess: Vec<u8>) {
        // u can do something here
        self.handler_mess(mess).await;
    }
}

fn main() {
    let address = String::from("127.0.0.1:8080");
    let mut tc = TcpClientHandler::default();
    async_std::task::block_on(tc.on_read(address));
}
