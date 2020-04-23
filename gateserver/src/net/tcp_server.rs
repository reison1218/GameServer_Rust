use super::*;
use crate::ID;
use protobuf::ProtobufEnum;
use std::sync::atomic::Ordering;
use tools::tcp::TcpSender;
use tools::cmd_code::GameCode::LineOff;
use tools::cmd_code::RoomCode;


struct TcpServerHandler {
    pub tcp: Option<TcpSender>, //相当于channel
    pub add: Option<String>,     //客户端地址
    cm: Arc<RwLock<ChannelMgr>>, //channel管理器
}

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

impl tools::tcp::Handler for TcpServerHandler {
    fn try_clone(&self) -> Self {
        let mut tcp: Option<TcpSender> = None;
        if self.tcp.is_some() {
            tcp = Some(self.tcp.as_ref().unwrap().clone());
        }

        TcpServerHandler {
            tcp: tcp,
            add: self.add.clone(),
            cm: self.cm.clone(),
        }
    }

    fn on_open(&mut self, sender: TcpSender) {
        self.tcp = Some(sender);
    }

    fn on_close(&mut self) {
        info!(
            "tcp_server:客户端断开连接,通知其他服卸载玩家数据:{}",
            self.add.as_ref().unwrap()
        );

        let token = self.tcp.as_ref().unwrap().token;
        let mut write = self.cm.write().unwrap();
        write.off_line(token);
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        info!("GateServer got message '{:?}'. ", mess);
        // let mut mp = vec_to_message(&mess[..]);
        // self.handle_binary(mp);

        //以下代码做性能测试
        let mut mp = MessPacketPt::new();
        let mut s_l = C_USER_LOGIN::new();
        s_l.set_avatar("test".to_owned());
        s_l.set_nickName("test".to_owned());
        {
            ID.write().unwrap().id += 1;
            let id = ID.write().unwrap().id;
            s_l.set_userId(id);
            mp.set_user_id(id);
        }

        mp.set_cmd(GameCode::Login as u32);
        mp.set_is_broad(false);
        mp.set_is_client(true);
        let result = s_l.write_to_bytes();
        if result.is_err() {
            error!("protobuf转换错误：{:?}", result.err().unwrap());
            return;
        }
        mp.set_data(result.unwrap());
        self.handle_binary(mp);
        //如果是文本数据
    }
}

impl TcpServerHandler {
    ///处理二进制数据
    fn handle_binary(&mut self, mut mess: MessPacketPt) {
        let token = self.tcp.as_ref().unwrap().token;
        let mut write = self.cm.write().unwrap();
        let user_id = write.get_channels_user_id(&token);

        //如果内存不存在数据，请求的命令又不是登录命令,则判断未登录异常操作
        if user_id.is_none() && mess.get_cmd() != GameCode::Login as u32 {
            error!(
                "this player is not login!cmd:{},token:{}",
                mess.get_cmd(),
                token
            );
            return;
        }
        //执行登录
        if mess.get_cmd() == GameCode::Login as u32 {
            let mut c_login = C_USER_LOGIN::new();
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
            write.add_gate_user(
                c_login.get_userId(),
                None,
                Some(self.tcp.as_ref().unwrap().clone()),
            );
        }
        //封装packet转发到其他服
        let user_id = write.get_channels_user_id(&token);
        mess.set_user_id(*user_id.unwrap());
        //释放write指针，绕过编译器检查
        std::mem::drop(write);
        //转发函数
        self.arrange_packet(mess);
    }

    ///数据包转发
    fn arrange_packet(&mut self, mess: MessPacketPt) {
        let mut write = self.cm.write().unwrap();
        //转发到游戏服
        if mess.get_cmd() >= GameCode::Min as u32  && mess.get_cmd() <= GameCode::Max as u32 {
            write.write_to_game(mess);
            return;
        }
        //转发到房间服
        if mess.get_cmd() >= RoomCode::Min as u32 && mess.get_cmd() <=  RoomCode::Max as u32{
            write.write_to_room(mess);
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
    tools::tcp::tcp_server::new(address,sh);
}
