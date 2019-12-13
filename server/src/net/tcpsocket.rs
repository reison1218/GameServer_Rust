use super::*;
use std::io::Write;
use chrono::Duration;

struct TcpServer {}

pub async fn new(game_mgr: Arc<RwLock<GameMgr>>) {
    let tl = std::net::TcpListener::bind("127.0.0.1:8888").unwrap();
    //设置非阻塞
    // tl.set_nonblocking(true).expect("Cannot set non-blocking");

    let net_pool = ThreadPool::with_name("net_thread_pool".to_owned(), 4);

    info!("开始监听8888端口！");

    for stream in tl.incoming() {
        match stream {
            Ok(mut s) => {
                let cl = move || loop {
                    let mut bytes: [u8; 1024] = [0; 1024];
                    let size = s.read(&mut bytes).unwrap();
                    if size == 0 {
                        continue;
                    }
                    info!("读取到gate数据,数据长度:{}", size);
                    let mut bytes = ByteBuf::from(&bytes);
                    let len = bytes.read_u32();
                    let cmd = bytes.read_u32();
                    let pd = PacketDes::new(cmd);
                    let packet = Packet::new(pd);

                };
                net_pool.execute(cl);
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
    }
}

fn read_handler() {}
