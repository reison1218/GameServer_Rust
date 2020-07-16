use super::*;
use tools::cmd_code::RoomCode;
use tools::protos::room::S_ROOM_MEMBER_LEAVE_NOTICE;

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
        let address: Option<&str>;
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
        println!("=========================================={}", mess.len());
        let packet_array = Packet::build_array_from_server(mess);

        if let Err(e) = packet_array {
            error!("{:?}", e.to_string());
            return;
        }
        let packet_array = packet_array.unwrap();

        for mut packet in packet_array {
            println!(
                "******************************************接收到其他服信息,user_id:{},cmd:{}",
                packet.get_user_id(),
                packet.get_cmd(),
            );
            //判断是否是发给客户端消息
            if packet.is_client() && packet.get_cmd() > 0 {
                let mut write = self.cp.write().unwrap();
                let gate_user = write.get_mut_user_channel_channel(&packet.get_user_id());
                match gate_user {
                    Some(user) => {
                        user.get_tcp_mut_ref().write(packet.build_client_bytes());
                    }
                    None => {
                        warn!(
                            "user data is null,id:{},cmd:{}",
                            &packet.get_user_id(),
                            packet.get_cmd()
                        );
                        return;
                    }
                }
            } else {
                //判断是否要转发到其他服务器进程消息
                self.arrange_packet(packet);
            }
        }
    }

    fn get_address(&self) -> &str {
        let address = CONF_MAP.get_str("gamePort");
        address
    }
}
