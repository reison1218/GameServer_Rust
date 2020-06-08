use super::*;

use std::io::Write;
use std::sync::Arc;
use tools::cmd_code::{GameCode, RoomCode};
use tools::tcp::TcpSender;

///channel管理结构体
pub struct ChannelMgr {
    //游戏服tcpstream
    pub game_client_channel: Option<TcpStream>,
    //房间服stream
    pub room_client_channel: Option<TcpStream>,
    //玩家channels
    pub user_channel: HashMap<u32, GateUser>,
    //token,user_id
    pub channels: HashMap<usize, u32>,
}

impl ChannelMgr {
    ///创建channelmgr结构体
    pub fn new() -> Self {
        let players: HashMap<u32, GateUser> = HashMap::new();
        let cm = ChannelMgr {
            game_client_channel: None,
            room_client_channel: None,
            user_channel: players,
            channels: HashMap::new(),
        };
        cm
    }

    pub fn set_game_client_channel(&mut self, ts: TcpStream) {
        self.game_client_channel = Some(ts);
    }

    pub fn set_room_client_channel(&mut self, ts: TcpStream) {
        self.room_client_channel = Some(ts);
    }

    ///处理离线事件
    /// token：sender唯一标识
    pub fn off_line(&mut self, token: usize) {
        let user_id = self.get_channels_user_id(&token);
        match user_id {
            Some(user_id) => {
                let user_id = *user_id;
                self.notice_off_line(user_id, &token);
            }
            None => {
                warn!("user_id is none for token:{},so nothing to do!", token);
            }
        }
    }

    ///通知下线
    fn notice_off_line(&mut self, user_id: u32, token: &usize) {
        //初始化包
        let mut packet = Packet::default();
        packet.set_user_id(user_id);
        packet.set_len(16 as u32);
        packet.set_is_client(false);
        packet.set_is_broad(false);
        //发给游戏服
        packet.set_cmd(GameCode::LineOff as u32);
        self.write_to_game(packet.clone());
        //发给房间服
        packet.set_cmd(RoomCode::LineOff as u32);
        self.write_to_room(packet);
        //关闭连接
        self.close_remove(token);
    }

    ///写到游戏服
    pub fn write_to_game(&mut self, packet: Packet) {
        if self.game_client_channel.is_none() {
            error!("disconnect with Game-Server,pls connect Game-Server before send packet!");
            return;
        }
        let gc = self.game_client_channel.as_mut().unwrap();
        let size = gc.write(&packet.build_server_bytes()[..]);
        match size {
            Ok(s) => {
                info!("write to server size:{}", s);
                let res = gc.flush();
                if res.is_err() {
                    error!("flush has error!mess:{:?}", res.err().unwrap().to_string());
                }
            }
            Err(e) => {
                error!("{:?}", e.to_string());
                return;
            }
        }
    }

    ///写到房间服
    #[warn(unused_must_use)]
    pub fn write_to_room(&mut self, packet: Packet) {
        if self.room_client_channel.is_none() {
            error!("disconnect with Room-Server,pls connect Room-Server before send packet!");
            return;
        }
        let rc = self.room_client_channel.as_mut().unwrap();
        let size = rc.write(&packet.build_server_bytes()[..]);
        match size {
            Ok(s) => {
                info!("write to server size:{}", s);
                let res = rc.flush();
                if res.is_err() {
                    error!("{:?}", res.err().unwrap().to_string());
                }
            }
            Err(e) => {
                error!("{:?}", e.to_string());
                return;
            }
        }
    }

    //添加gateuser
    pub fn add_gate_user(
        &mut self,
        user_id: u32,
        ws: Option<Arc<WsSender>>,
        tcp: Option<TcpSender>,
    ) {
        let mut token = 0;
        if ws.is_some() {
            token = ws.as_ref().unwrap().token().0;
        }
        if tcp.is_some() {
            token = tcp.as_ref().unwrap().token;
        }
        self.insert_channels(token, user_id);
        self.insert_user_channel(user_id, GateUser::new(user_id, ws, tcp));
    }

    ///插入channel,key：userid,v:channel
    pub fn insert_user_channel(&mut self, token: u32, gate_user: GateUser) {
        self.user_channel.insert(token, gate_user);
    }
    ///插入token-userid的映射
    pub fn insert_channels(&mut self, token: usize, user_id: u32) {
        self.channels.insert(token, user_id);
    }
    ///获得玩家channel k:userid
    pub fn get_user_channel(&mut self, user_id: &u32) -> Option<&GateUser> {
        self.user_channel.get(user_id)
    }

    ///根据token获得userid
    pub fn get_channels_user_id(&mut self, token: &usize) -> Option<&u32> {
        self.channels.get(token)
    }

    ///通过userid获得channel
    pub fn get_mut_user_channel_channel(&mut self, user_id: &u32) -> Option<&mut GateUser> {
        self.user_channel.get_mut(user_id)
    }

    ///关闭channel句柄，并从内存中删除
    pub fn close_remove(&mut self, token: &usize) {
        let user_id = self.channels.remove(token);
        if user_id.is_none() {
            info!("channel_mgr:user_id is none for token:{}", token);
            return;
        }
        let user_id = &user_id.unwrap();
        let gate_user = self.user_channel.get_mut(user_id);
        if gate_user.is_none() {
            info!("channel_mgr:gate_user is none for user_id:{}", user_id);
            return;
        }
        gate_user.unwrap().close();
        self.user_channel.remove(user_id);
        info!("channel_mgr:玩家断开连接，关闭句柄释放资源：{}", user_id);
    }

    ///T掉所有玩家
    pub fn kick_all(&mut self) {
        let res = self.channels.clone();
        for (token, user_id) in res.iter() {
            self.notice_off_line(*user_id, token);
        }
    }
}
