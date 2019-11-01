use super::*;
use threadpool::ThreadPool;

struct TcpServer {}

pub fn new() {
    let tl = std::net::TcpListener::bind("127.0.0.1:8888").unwrap();
    //设置非阻塞
    // tl.set_nonblocking(true).expect("Cannot set non-blocking");

    let net_pool = ThreadPool::with_name("net_thread_pool".to_owned(), 4);

    info!("开始监听8888端口！");

    for stream in tl.incoming() {
        match stream {
            Ok(mut s) => {
                let cl = move || loop {
                    let mut bytes: [u8; 512] = [0; 512];
                    let size = s.read(&mut bytes).unwrap();
                    if size == 0 {
                        continue;
                    }
                    info!("读取到gate数据{}", size);
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
