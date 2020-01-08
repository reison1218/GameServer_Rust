use super::*;
use chrono::Duration;
use std::io::Write;

pub struct TcpServer {}

pub async fn new(game_mgr: Arc<RwLock<GameMgr>>) {
    let tl = std::net::TcpListener::bind("127.0.0.1:8888").unwrap();
    //设置非阻塞
    // tl.set_nonblocking(true).expect("Cannot set non-blocking");

    info!("开始监听8888端口！");

    //let mut ts: Option<TcpStream> = None;
    for stream in tl.incoming() {
        match stream {
            Ok(mut s) => {
                let cl = move || loop {
                    let mut bytes: [u8; 1024] = [0; 1024];
                    // ts = Some(s);
                    let size = s.read(&mut bytes).unwrap();

                    if size == 0 {
                        continue;
                    }
                    info!("读取到gate数据,数据长度:{}", size);
                    let mut bb = ByteBuf::new();
                    let mut mess = MessPacketPt::new();
                    mess.merge_from_bytes(&bytes);
                    let mut packet = build_packet(mess);
                };
                &THREAD_POOL.submit_game(cl);
            }
            Err(e) => {
                println!("{}", e.to_string());
            }
        }
    }
}
///byte数组转换Packet
pub fn build_packet(mess: MessPacketPt) -> Packet {
    //封装成packet
    let mut pd = PacketDes::new(mess.cmd);
    let mut packet = Packet::new(pd);
    packet.set_bytes(&mess.write_to_bytes().unwrap()[..]);
    packet
}

fn read_handler() {}
