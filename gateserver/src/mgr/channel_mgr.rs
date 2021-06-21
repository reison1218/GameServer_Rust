use crate::entity::gateuser::GateUser;
use crate::net::http::notice_user_center;
use crate::net::http::UserCenterNoticeType;
use crossbeam::channel::Sender;
use log::warn;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tools::cmd_code::RoomCode;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;
use ws::Sender as WsSender;

///channel管理结构体
pub struct ChannelMgr {
    //游戏服tcpstream
    pub game_client_channel: Option<Sender<Vec<u8>>>,
    //房间服stream
    // pub room_client_channel: Option<TcpStream>,
    //游戏中心stream
    pub game_center_client_channel: Option<Sender<Vec<u8>>>,
    //玩家channels user_id GateUser
    pub user_channel: HashMap<u32, GateUser>,
    //token,user_id
    pub channels: HashMap<usize, u32>,
    //临时会话map
    pub temp_channels: HashMap<u32, Option<TcpSender>>,
}

impl ChannelMgr {
    ///创建channelmgr结构体
    pub fn new() -> Self {
        let players: HashMap<u32, GateUser> = HashMap::new();
        let cm = ChannelMgr {
            game_client_channel: None,
            // room_client_channel: None,
            game_center_client_channel: None,
            user_channel: players,
            channels: HashMap::new(),
            temp_channels: HashMap::new(),
        };
        cm
    }

    pub fn set_game_client_channel(&mut self, ts: Sender<Vec<u8>>) {
        self.game_client_channel = Some(ts);
    }

    // pub fn set_room_client_channel(&mut self, ts: TcpStream) {
    //     self.room_client_channel = Some(ts);
    // }

    pub fn set_game_center_client_channel(&mut self, ts: Sender<Vec<u8>>) {
        self.game_center_client_channel = Some(ts);
    }

    ///处理离线事件
    /// token：sender唯一标识
    pub fn off_line(&mut self, token: usize) {
        let user_id = self.get_channels_user_id(&token);
        match user_id {
            Some(user_id) => {
                let user_id = *user_id;
                self.notice_off_line(user_id);
                //关闭连接
                self.close_remove(&token);

                info!("tcp_server:客户端断开连接,通知其他服卸载玩家数据");
            }
            None => {
                //warn!("user_id is none for token:{},so nothing to do!", token);
            }
        }
    }

    ///通知下线
    fn notice_off_line(&mut self, user_id: u32) {
        let cmd = RoomCode::OffLine.into_u32();

        //初始化包
        let mut packet = Packet::default();
        packet.set_user_id(user_id);
        packet.set_len(14_u32);
        packet.set_is_client(false);
        packet.set_is_broad(false);
        packet.set_cmd(cmd);
        //发给房间相关服
        self.write_to_game_center(packet);
        //通知用户中心
        async_std::task::spawn(notice_user_center(user_id, UserCenterNoticeType::OffLine));
    }

    ///写到游戏服
    pub fn write_to_game(&mut self, packet: Packet) {
        if self.game_client_channel.is_none() {
            error!("disconnect with Game-Server,pls connect Game-Server before send packet!");
            return;
        }
        let gc = self.game_client_channel.as_mut().unwrap();
        let size = gc.send(packet.build_server_bytes());
        if let Err(e) = size {
            error!("{:?}", e);
        }
    }

    ///停服
    pub fn stop_server(&mut self) {
        self.kick_all();
    }

    pub fn kick_player(&mut self, user_id: u32) -> bool {
        let gate_user = self.get_user_channel(&user_id);
        if let None = gate_user {
            return false;
        }
        let gate_user = gate_user.unwrap();
        let token = gate_user.get_token();
        self.off_line(token);
        true
    }

    ///写到游戏中心服
    pub fn write_to_game_center(&mut self, packet: Packet) {
        if self.game_center_client_channel.is_none() {
            error!("disconnect with Game-Center,pls connect Game-Center before send packet!");
            return;
        }
        let rc = self.game_center_client_channel.as_mut().unwrap();
        let size = rc.send(packet.build_server_bytes());
        if let Err(e) = size {
            error!("{:?}", e);
        }
    }

    ///将临时到tcpsender转化到gateuser
    pub fn temp_channel_2_gate_user(&mut self, user_id: u32) {
        let res = self.temp_channels.remove(&user_id);
        if let None = res {
            warn!(
                "temp_channels could not find tcpsender for user_id:{}",
                user_id
            );
            return;
        }
        let res = res.unwrap();
        self.add_gate_user(user_id, None, res);
        //通知用户中心
        async_std::task::spawn(notice_user_center(user_id, UserCenterNoticeType::Login));
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
        self.insert_user_channel(user_id, GateUser::new(ws, tcp));
    }

    ///插入channel,key：userid,v:channel
    pub fn insert_user_channel(&mut self, user_id: u32, gate_user: GateUser) {
        self.user_channel.insert(user_id, gate_user);
    }
    ///插入token-userid的映射
    pub fn insert_channels(&mut self, token: usize, user_id: u32) {
        self.channels.insert(token, user_id);
    }
    ///获得玩家channel k:userid
    pub fn get_user_channel(&self, user_id: &u32) -> Option<&GateUser> {
        self.user_channel.get(user_id)
    }

    ///根据token获得userid
    pub fn get_channels_user_id(&self, token: &usize) -> Option<&u32> {
        self.channels.get(token)
    }

    ///通过userid获得channel
    pub fn get_mut_user_channel(&mut self, user_id: &u32) -> Option<&mut GateUser> {
        self.user_channel.get_mut(user_id)
    }

    ///关闭channel句柄，并从内存中删除
    pub fn close_remove(&mut self, token: &usize) {
        let user_id = self.channels.remove(token);
        if user_id.is_none() {
            return;
        }
        let user_id = &user_id.unwrap();
        let gate_user = self.user_channel.get_mut(user_id);
        if let Some(gate_user) = gate_user {
            gate_user.close();
        }
        self.user_channel.remove(user_id);
        self.temp_channels.remove(user_id);
        info!("channel_mgr:玩家断开连接，关闭句柄释放资源：{}", user_id);
    }

    ///T掉所有玩家
    pub fn kick_all(&mut self) {
        let res = self.channels.clone();
        for (_, &user_id) in res.iter() {
            self.kick_player(user_id);
        }
        info!("kick all finish!");
    }
}
