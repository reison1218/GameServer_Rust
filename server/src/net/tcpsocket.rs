use super::*;
use threadpool::ThreadPool;

struct TcpServer {}

pub fn new() {
    let mut tl = std::net::TcpListener::bind("127.0.0.1:8888").unwrap();
    //设置非阻塞
    tl.set_nonblocking(true);
    let mut ts = tl.accept().unwrap();

    let mut net_pool = ThreadPool::new_with_name("net_thread_pool".to_owned(), 4);

    info!("开始监听8888端口！");
    let mut cl = move || loop {
        let mut bytes: [u8; 512] = [0; 512];
        let size = ts.0.read(&mut bytes).unwrap();
        if size == 0 {
            continue;
        }
        info!("读取到gate数据{}", size);
    };
    net_pool.execute(cl);
}

fn read_handler() {}
