use super::*;
use crate::net::http::notice_user_center;
use async_std::sync::RwLock;
use async_std::sync::RwLockWriteGuard;
use async_std::task::block_on;
use async_trait::async_trait;
use chrono::Local;
use tools::cmd_code::{ClientCode, RoomCode};
use tools::protos::protocol::HEART_BEAT;
use tools::tcp::TcpSender;

struct TcpServerHandler {
    pub tcp: Option<TcpSender>,  //相当于channel
    pub add: Option<String>,     //客户端地址
    cm: Arc<RwLock<ChannelMgr>>, //channel管理器
}

tools::get_mut_ref!(TcpServerHandler);

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

#[async_trait]
impl tools::tcp::Handler for TcpServerHandler {
    async fn try_clone(&self) -> Self {
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

    async fn on_open(&mut self, sender: TcpSender) {
        self.tcp = Some(sender);
    }

    async fn on_close(&mut self) {
        info!(
            "tcp_server:客户端断开连接,通知其他服卸载玩家数据:{}",
            self.add.as_ref().unwrap()
        );

        let token = self.tcp.as_ref().unwrap().token;
        let mut lock = self.cm.write().await;
        lock.off_line(token);
    }

    async fn on_message(&mut self, mess: Vec<u8>) {
        //校验包长度
        if mess.is_empty() || mess.len() < 16 {
            error!("client packet len is wrong!");
            return;
        }
        let packet_array = Packet::build_array_from_client(mess);

        if packet_array.is_err() {
            error!("{:?}", packet_array.err().unwrap().to_string());
            return;
        }
        let packet_array = packet_array.unwrap();

        for packet in packet_array {
            let cmd = packet.get_cmd();
            info!("GateServer receive data of client!cmd:{}", cmd);
            self.handle_binary(packet);
        }
    }
}

impl TcpServerHandler {
    ///写到客户端
    fn write_to_client(&mut self, bytes: Vec<u8>) {
        let res = self.tcp.as_mut();
        match res {
            Some(ts) => {
                ts.write(bytes);
            }
            None => {
                warn!("TcpServerHandler's tcp is None!");
            }
        }
    }

    ///处理二进制数据
    fn handle_binary(&mut self, mut packet: Packet) {
        let token = self.tcp.as_ref().unwrap().token;
        let mut lock = block_on(self.cm.write());
        let user_id = lock.get_channels_user_id(&token);

        //如果内存不存在数据，请求的命令又不是登录命令,则判断未登录异常操作
        if user_id.is_none() && packet.get_cmd() != GameCode::Login.into_u32() {
            let str = format!(
                "this player is not login!cmd:{},token:{}",
                packet.get_cmd(),
                token
            );
            warn!("{:?}", str.as_str());
        }

        let u_id;
        //执行登录
        if packet.get_cmd() == GameCode::Login.into_u32() {
            let mut c_u_l = C_USER_LOGIN::new();
            let res = c_u_l.merge_from_bytes(packet.get_data());
            if res.is_err() {
                error!("{:?}", res.err().unwrap().to_string());
                return;
            }
            u_id = c_u_l.get_user_id();
            let res = handle_login(packet.get_data(), &mut lock);
            if let Err(e) = res {
                let str = e.to_string();
                let mut sul = S_USER_LOGIN::new();
                sul.set_is_succ(false);
                sul.set_err_mess(str.clone());
                packet.set_cmd(ClientCode::Login as u32);
                packet.set_data_from_vec(sul.write_to_bytes().unwrap());
                std::mem::drop(lock);
                self.write_to_client(packet.build_client_bytes());
                return;
            }
            lock.add_gate_user(u_id, None, self.tcp.clone());
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

        if packet.get_cmd() == ClientCode::HeartBeat.into_u32() {
            let mut hb = HEART_BEAT::new();
            let time_stamp = Local::now().timestamp() as u64;
            hb.set_sys_time(time_stamp);
            let bytes = hb.write_to_bytes().unwrap();

            let res =
                Packet::build_packet_bytes(ClientCode::HeartBeat.into(), u_id, bytes, false, true);
            let gate_user = lock.user_channel.get_mut(&u_id);
            if let None = gate_user {
                return;
            }
            let gate_user = gate_user.unwrap();
            gate_user.get_tcp_mut_ref().write(res);
            info!(
                "回给客户端消息,user_id:{},cmd:{}",
                packet.get_user_id(),
                packet.get_cmd(),
            );
        }
        std::mem::drop(lock);
        //转发函数
        self.arrange_packet(packet);
    }

    ///数据包转发
    fn arrange_packet(&mut self, packet: Packet) {
        let mut lock = block_on(self.cm.write());
        //转发到游戏服
        if packet.get_cmd() >= GameCode::Min as u32 && packet.get_cmd() <= GameCode::Max as u32 {
            lock.write_to_game(packet);
            return;
        }
        //转发到房间服
        if packet.get_cmd() >= RoomCode::Min as u32 && packet.get_cmd() <= RoomCode::Max as u32 {
            lock.write_to_room(packet);
            return;
        }
    }
}

///创建新的tcpserver并开始监听
pub fn new(address: &str, cm: Arc<RwLock<ChannelMgr>>) {
    let sh = TcpServerHandler {
        tcp: None,
        cm,
        add: Some(address.to_string()),
    };
    let res = tools::tcp::tcp_server::new(address, sh);
    let res = block_on(res);
    if res.is_err() {
        error!("{:?}", res.err().unwrap().to_string());
        std::process::abort();
    }
}

///处理登陆逻辑
fn handle_login(bytes: &[u8], lock: &mut RwLockWriteGuard<ChannelMgr>) -> anyhow::Result<()> {
    let mut c_login = C_USER_LOGIN::new();
    c_login.merge_from_bytes(bytes)?;
    //校验用户中心账号是否已经登陆了
    let uc_res = check_uc_online(&c_login.get_user_id())?;
    //校验内存
    let mem_res = check_mem_online(&c_login.get_user_id(), lock);
    //如果用户中心登陆了或者本地内存登陆了，直接错误返回
    if uc_res || mem_res {
        // modify_redis_user(c_login.get_user_id(), false);
        let str = format!(
            "this account already login!uc_res:{},mem_res:{},user_id:{}",
            uc_res,
            mem_res,
            &c_login.get_user_id()
        );
        warn!("{:?}", str.as_str());
        anyhow::bail!("{:?}", str)
    }
    Ok(())
}
