use super::*;

pub struct ChannelMgr {
    game_channel: TcpStream,
}

impl ChannelMgr {
    pub fn new() -> ChannelMgr {
        let mut game_channel = new_tcp_client();
        let mut cm = ChannelMgr {
            game_channel: game_channel,
        };
        cm.connect_game();
        cm
    }
    pub fn connect_game(&mut self) {
        let mut v: [u8; 512] = [0; 512];
        self.game_channel.read(&mut v);
        info!("连接GameServer成功！");
    }
    pub fn connect_room() {}
}

pub fn new_tcp_client() -> TcpStream {
    let mut ts = TcpStream::connect("127.0.0.1:8888").unwrap();
    //设置非阻塞
    ts.set_nonblocking(true);
    //不组包
    ts.set_nodelay(true);
    ts
}
