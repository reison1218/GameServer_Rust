use super::*;
use crate::ID;
use protobuf::ProtobufEnum;
use std::sync::atomic::Ordering;
use tcp::tcp::MySyncSender;

#[derive(Clone)]
struct TcpServerHandler {
    pub tcp: Option<MySyncSender>, //相当于channel
    pub add: Option<String>,       //客户端地址
    cm: Arc<RwLock<ChannelMgr>>,   //channel管理器
}

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

impl tcp::tcp::Handler for TcpServerHandler {
    fn on_open(&mut self, sender: MySyncSender) {
        self.tcp = Some(sender);
    }

    fn on_close(&mut self) {
        info!(
            "客户端断开连接,通知游戏服卸载玩家数据{}",
            self.add.as_ref().unwrap()
        );
        //self.tcp.as_ref().unwrap().sender.send(Packet::new(1));

        let token = self.tcp.as_ref().unwrap().token;
        let mut write = self.cm.write().unwrap();
        let mut packet = Packet::new(101);
        let user_id = write.get_channels_user_id(&token);
        if user_id.is_none() {
            return;
        }

        packet.set_user_id(*user_id.unwrap());
        write.write_to_game(packet);
        write.close_remove(&token);
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        info!("GateServer got message '{:?}'. ", mess);

        //如果是二进制数据
        let bytes = &mess[..];
        //self.handle_binary(bytes);
        //此处代码做性能测试
        let mut mp = crate::protos::base::MessPacketPt::new();
        let mut s_l = crate::protos::protocol::C_USER_LOGIN::new();
        s_l.set_avatar("test".to_owned());
        s_l.set_nickName("test".to_owned());
        {
            ID.write().unwrap().id += 1;
        }
        let id = ID.write().unwrap().id;

        s_l.set_userId(id);
        mp.set_cmd(1002);
        let result = s_l.write_to_bytes();
        if result.is_err() {
            error!("protobuf转换错误：{:?}", result.err().unwrap());
            return;
        }
        mp.set_data(result.unwrap());
        let bytes = &mp.write_to_bytes().unwrap()[..];
        self.handle_binary(bytes);
        //如果是文本数据
    }
}

impl TcpServerHandler {
    fn handle_binary(&mut self, bytes: &[u8]) {
        let mut mess = MessPacketPt::new();
        let result = mess.merge_from_bytes(bytes);
        if result.is_err() {
            error!("protobuf转换错误：{:?}", result.err().unwrap());
            return;
        }

        let token = self.tcp.as_ref().unwrap().token;
        let mut write = self.cm.write().unwrap();
        let user_id = write.get_channels_user_id(&token);

        //如果内存不存在数据，请求的命令又不是登录命令,则判断未登录异常操作
        if user_id.is_none() && mess.get_cmd() != C_USER_LOGIN.value() as u32 {
            error!(
                "this player is not login!cmd:{},token:{}",
                mess.get_cmd(),
                token
            );
            return;
        }
        //执行登录
        if mess.get_cmd() == C_USER_LOGIN.value() as u32 {
            let mut c_login = C_USER_LOGIN_PROTO::new();
            let result = c_login.merge_from_bytes(mess.get_data());
            if result.is_err() {
                error!("protobuf转换错误：{:?}", result.err().unwrap());
                return;
            }
            let mut gate_user = write.get_mut_user_channel_channel(&c_login.get_userId());
            //如果有，则执行T下线
            if gate_user.is_some() {
                let token = gate_user.as_mut().unwrap().get_tcp_ref().token;
                //释放可变指针，免得出现重复可变指针编译不通过
                std::mem::drop(gate_user.unwrap());
                write.close_remove(&token);
            }
            write.add_gate_user(c_login.get_userId(), None, Some(self.tcp.clone().unwrap()));
        }
        //封装packet转发到其他服
        let mut packet = Packet::new(mess.get_cmd());
        let user_id = write.get_channels_user_id(&token);
        packet.set_user_id(*user_id.unwrap());
        packet.set_data(mess.get_data());
        //释放write指针，绕过编译器检查
        std::mem::drop(write);
        //转发函数
        self.arrange_packet(packet);
    }

    ///数据包转发
    fn arrange_packet(&mut self, packet: Packet) {
        //转发到游戏服
        if packet.get_cmd() >= GAME_MIN && packet.get_cmd() <= GAME_MAX {
            let mut write = self.cm.write().unwrap();
            write.write_to_game(packet);
            return;
        }
        //转发到房间服
        if packet.get_cmd() >= ROOM_MIN && packet.get_cmd() <= ROOM_MAX {
            return;
        }
    }
}

pub fn new(address: &str, cm: Arc<RwLock<ChannelMgr>>) {
    let sh = TcpServerHandler {
        tcp: None,
        cm: cm,
        add: Some(address.to_string()),
    };
    let mut tcp_server = tcp::tcp::tcp_server::new(address, sh).unwrap();
    tcp_server.on_listen();
}
