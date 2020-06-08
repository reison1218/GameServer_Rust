use super::*;
use crate::net::http::notice_user_center;
use tools::cmd_code::{ClientCode, RoomCode};
use tools::tcp::TcpSender;

struct TcpServerHandler {
    pub tcp: Option<TcpSender>,  //相当于channel
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
        //校验包长度
        if mess.is_empty() || mess.len() < 16 {
            error!("client packet len is wrong!");
            return;
        }
        let packet = Packet::from_only_client(mess);
        match packet {
            Ok(p) => {
                let cmd = p.get_cmd();
                info!("GateServer receive data of client!cmd:{}", cmd);
                self.handle_binary(p.clone());
            }
            Err(e) => {
                error!("{:?}", e);
            }
        }
    }
}

impl TcpServerHandler {
    ///写到客户端
    fn write_to_client(&mut self, bytes: Vec<u8>) {
        let res = self.tcp.as_mut();
        match res {
            Some(ts) => {
                let res = ts.write(bytes);
                if res.is_err() {
                    error!("write error!mess:{:?}", res.err().unwrap().to_string());
                }
            }
            None => {
                warn!("TcpServerHandler's tcp is None!");
            }
        }
    }

    ///处理二进制数据
    fn handle_binary(&mut self, mut packet: Packet) {
        let token = self.tcp.as_ref().unwrap().token;
        let mut write = self.cm.write().unwrap();
        let user_id = write.get_channels_user_id(&token);

        //如果内存不存在数据，请求的命令又不是登录命令,则判断未登录异常操作
        if user_id.is_none() && packet.get_cmd() != GameCode::Login as u32 {
            let str = format!(
                "this player is not login!cmd:{},token:{}",
                packet.get_cmd(),
                token
            );
            warn!("{:?}", str.as_str());
        }

        let u_id;
        //执行登录
        if packet.get_cmd() == GameCode::Login as u32 {
            let mut c_u_l = C_USER_LOGIN::new();
            let res = c_u_l.merge_from_bytes(packet.get_data());
            if res.is_err() {
                error!("{:?}", res.err().unwrap().to_string());
                return;
            }
            u_id = c_u_l.get_user_id();
            let res = handle_login(packet.get_data(), &mut write);
            if res.is_err() {
                let str = res.err().unwrap().to_string();
                let mut res = S_USER_LOGIN::new();
                res.set_is_succ(false);
                res.set_err_mess(str.clone());
                packet.set_cmd(ClientCode::Login as u32);
                packet.set_data_from_vec(res.write_to_bytes().unwrap());
                std::mem::drop(write);
                self.write_to_client(packet.build_client_bytes());
                return;
            }
            write.add_gate_user(u_id, None, self.tcp.clone());
            //通知用户中心
            async_std::task::spawn(notice_user_center(u_id, "login"));
        } else if user_id.is_none() {
            let str = format!("this user_id is invalid!user_id:{}", packet.get_user_id());
            warn!("{:?}", str.as_str());
            return;
        } else {
            u_id = *user_id.unwrap();
        }
        packet.set_user_id(u_id);
        std::mem::drop(write);
        //转发函数
        self.arrange_packet(packet);
    }

    ///数据包转发
    fn arrange_packet(&mut self, packet: Packet) {
        let mut write = self.cm.write().unwrap();
        //转发到游戏服
        if packet.get_cmd() >= GameCode::Min as u32 && packet.get_cmd() <= GameCode::Max as u32 {
            write.write_to_game(packet);
            return;
        }
        //转发到房间服
        if packet.get_cmd() >= RoomCode::Min as u32 && packet.get_cmd() <= RoomCode::Max as u32 {
            write.write_to_room(packet);
            return;
        }
    }
}

///创建新的tcpserver并开始监听
pub fn new(address: &str, cm: Arc<RwLock<ChannelMgr>>) {
    let sh = TcpServerHandler {
        tcp: None,
        cm: cm,
        add: Some(address.to_string()),
    };
    let res = tools::tcp::tcp_server::new(address, sh);
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
        std::process::abort();
    }
}

///处理登陆逻辑
fn handle_login(bytes: &[u8], write: &mut RwLockWriteGuard<ChannelMgr>) -> anyhow::Result<()> {
    let mut c_login = C_USER_LOGIN::new();
    c_login.merge_from_bytes(bytes)?;
    //校验用户中心账号是否已经登陆了
    let uc_res = check_uc_online(&c_login.get_user_id())?;
    //校验内存
    let mem_res = check_mem_online(&c_login.get_user_id(), write);
    //如果用户中心登陆了或者本地内存登陆了，直接错误返回
    if uc_res || mem_res {
        // modify_redis_user(c_login.get_user_id(), false);
        let str = format!(
            "this account already login!user_id:{}",
            &c_login.get_user_id()
        );
        anyhow::bail!("{:?}", str)
    }
    Ok(())
}
