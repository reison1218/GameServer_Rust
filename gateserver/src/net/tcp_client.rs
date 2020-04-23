use super::*;
use tools::cmd_code::{RoomCode, ClientCode};
use tools::tcp::TcpSender;

pub enum TcpClientType{
    GameServer,
    RoomServer,
}

pub struct TcpClientHandler {
    client_type:TcpClientType,
    ts: Option<TcpStream>,
    cp: Arc<RwLock<ChannelMgr>>,
}

impl TcpClientHandler {
    pub fn new(cp: Arc<RwLock<ChannelMgr>>,client_type:TcpClientType) -> TcpClientHandler {
        let mut tch = TcpClientHandler { ts: None, cp ,client_type};
        tch
    }

    ///数据包转发
    fn arrange_packet(&mut self, mess: MessPacketPt) {
        //转发到游戏服
        if mess.get_cmd() >= GameCode::Min as u32 && mess.get_cmd() <= GameCode::Max as u32 {
            let mut write = self.cp.write().unwrap();
            write.write_to_game(mess);
            return;
        }
        //转发到房间服
        if mess.get_cmd() >= RoomCode::Min as u32 && mess.get_cmd() <= RoomCode::Max as u32 {
            let mut write = self.cp.write().unwrap();
            write.write_to_room(mess);
            return;
        }
    }
}

impl ClientHandler for TcpClientHandler {
    fn on_open(&mut self, ts: TcpStream) {
        match self.client_type {
            TcpClientType::GameServer=>{
                self.cp.write().unwrap().game_client_channel = Some(ts.try_clone().unwrap());
            },
            TcpClientType::RoomServer=>{
                self.cp.write().unwrap().room_client_channel = Some(ts.try_clone().unwrap());
            }
        }
        self.ts = Some(ts);
    }

    fn on_close(&mut self) {
        let mut address:Option<&str> = None;
        match self.client_type {
            TcpClientType::GameServer=>{
                address = Some(CONF_MAP.get_str("gamePort"));
            },
            TcpClientType::RoomServer=>{
                address = Some(CONF_MAP.get_str("roomPort"));
            }
        }
        self.on_read(address.unwrap().to_string());
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        let mut mp = MessPacketPt::new();
        mp.merge_from_bytes(&mess[..]);
        //判断是否是发给客户端消息
        if mp.get_is_client() && mp.get_cmd() > 0 {
            info!("属于需要发给客户端的消息！");
            let mut write = self.cp.write().unwrap();
            let mut gate_user = write.get_mut_user_channel_channel(&mp.get_user_id());

            match gate_user {
                Some(user)=>{
                    user.get_tcp_mut_ref().write(mp.data);
                    info!("回客户端消息");
                },
                None=>{
                    error!("user data is null,id:{}", &mp.get_user_id());
                    return;
                }
            }

        } else { //判断是否要转发到其他服务器进程消息
            self.arrange_packet(mp);
        }
    }

    fn get_address(&self) -> &str {
        let address = CONF_MAP.get_str("gamePort");
        address
    }
}
