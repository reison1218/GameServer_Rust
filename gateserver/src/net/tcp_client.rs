use super::*;
use tools::cmd_code::{ClientCode, RoomCode};

pub enum TcpClientType {
    GameServer,
    RoomServer,
}

pub struct TcpClientHandler {
    client_type: TcpClientType,
    ts: Option<TcpStream>,
    cp: Arc<RwLock<ChannelMgr>>,
}

impl TcpClientHandler {
    pub fn new(cp: Arc<RwLock<ChannelMgr>>, client_type: TcpClientType) -> TcpClientHandler {
        let tch = TcpClientHandler {
            ts: None,
            cp,
            client_type,
        };
        tch
    }

    ///数据包转发
    fn arrange_packet(&mut self, packet: Packet) {
        //转发到游戏服
        if packet.get_cmd() >= GameCode::Min as u32 && packet.get_cmd() <= GameCode::Max as u32 {
            let mut write = self.cp.write().unwrap();
            write.write_to_game(packet);
            return;
        }
        //转发到房间服
        if packet.get_cmd() >= RoomCode::Min as u32 && packet.get_cmd() <= RoomCode::Max as u32 {
            let mut write = self.cp.write().unwrap();
            write.write_to_room(packet);
            return;
        }
    }
}

impl ClientHandler for TcpClientHandler {
    fn on_open(&mut self, ts: TcpStream) {
        match self.client_type {
            TcpClientType::GameServer => {
                self.cp
                    .write()
                    .unwrap()
                    .set_game_client_channel(ts.try_clone().unwrap());
            }
            TcpClientType::RoomServer => {
                self.cp
                    .write()
                    .unwrap()
                    .set_room_client_channel(ts.try_clone().unwrap());
            }
        }
        self.ts = Some(ts);
    }

    fn on_close(&mut self) {
        let mut address: Option<&str>;
        match self.client_type {
            TcpClientType::GameServer => {
                address = Some(CONF_MAP.get_str("game_port"));
            }
            TcpClientType::RoomServer => {
                address = Some(CONF_MAP.get_str("room_port"));
            }
        }
        self.on_read(address.unwrap().to_string());
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        let packet = Packet::from_only_server(mess);
        if packet.is_err() {
            error!("{:?}", packet.err().unwrap());
            return;
        }
        let mut packet = packet.unwrap();
        //判断是否是发给客户端消息
        if packet.is_client() && packet.get_cmd() > 0 {
            info!("属于需要发给客户端的消息！");
            let mut write = self.cp.write().unwrap();
            let gate_user = write.get_mut_user_channel_channel(&packet.get_user_id());

            match gate_user {
                Some(user) => {
                    let res = user.get_tcp_mut_ref().write(packet.build_client_bytes());
                    if res.is_err() {
                        error!("write error!mess:{:?}", res.err().unwrap().to_string());
                        return;
                    }
                    info!("回客户端消息,cmd:{}", packet.get_cmd());
                }
                None => {
                    error!("user data is null,id:{}", &packet.get_user_id());
                    return;
                }
            }
        } else {
            //判断是否要转发到其他服务器进程消息
            self.arrange_packet(packet);
        }
    }

    fn get_address(&self) -> &str {
        let address = CONF_MAP.get_str("gamePort");
        address
    }
}
