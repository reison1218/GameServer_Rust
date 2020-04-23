use tools::protos::protocol::S_USER_LOGIN;
use std::time::Duration;
use tools::util::packet::Packet;
use std::io::Write;
use protobuf::Message;
use std::net::TcpStream;
use tools::tcp::ClientHandler;
use tools::cmd_code::GameCode;

pub fn test_tcp_client(){
    for i in 0..50000{
        let m = move || {
            let mut tcp_client = TcpClientHandler::new();
            tcp_client.on_read("192.168.1.100:16801".to_string());
            //tcp_client.on_read("localhost:16801".to_string());
        };
        std::thread::spawn(m);
        std::thread::sleep(Duration::from_millis(100));
        println!("client:{}",i);
    }

    // let mut tcp_client = TcpClientHandler::new();
    // tcp_client.on_read("127.0.0.1:16801".to_string());
}
pub struct TcpClientHandler {
    ts: Option<TcpStream>,
}

impl TcpClientHandler {
    pub fn new() -> TcpClientHandler {
        let mut tch = TcpClientHandler { ts: None};
        tch
    }
}

impl ClientHandler for TcpClientHandler {
    fn on_open(&mut self, ts: TcpStream) {
        self.ts = Some(ts);
        let mut packet = Packet::new(GameCode::Login as u32);
        let mut s_l = tools::protos::protocol::C_USER_LOGIN::new();
        s_l.set_avatar("test".to_owned());
        s_l.set_nickName("test".to_owned());
        s_l.set_userId(2 as u32);
        packet.set_data(&s_l.write_to_bytes().unwrap()[..]);
        self.ts.as_mut().unwrap().write(&packet.all_to_vec()[..]).unwrap();
        self.ts.as_mut().unwrap().flush().unwrap();
    }

    fn on_close(&mut self) {
        println!("断开链接");
        let address = "192.168.1.100:16801";
        //let address = "localhost:16801";
        self.on_read(address.to_string());
    }

    fn on_message(&mut self, mess: Vec<u8>) {

        let mut s = S_USER_LOGIN::new();
        s.merge_from_bytes(&mess[..]);
        println!("from server:{:?}",s);
    }

    fn get_address(&self) -> &str {
        let address = "192.168.1.100:16801";
        //let address = "localhost:16801";
        address
    }
}