use super::*;
use async_std::sync::Mutex;
use async_std::task::block_on;
use tools::cmd_code::RoomCode;

#[warn(dead_code)]
pub struct ClientSender {
    pub user_id: Option<u32>,
    ws: WsSender,
}

///websockethandler
/// 监听websocket网络事件
pub struct WebSocketHandler {
    pub ws: Arc<WsSender>,          //相当于channel
    pub add: Option<String>,        //客户端地址
    pub cm: Arc<Mutex<ChannelMgr>>, //channel管理结构体指针
}

///实现相应的handler函数
impl Handler for WebSocketHandler {
    //关闭的时候调用
    fn on_shutdown(&mut self) {
        let res = self.ws.close(CloseCode::Invalid);
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
            return;
        }
        let token = self.ws.token().0;
        let mut lock = block_on(self.cm.lock());
        lock.close_remove(&token);
        let user_id = lock.get_channels_user_id(&token);
        let mut mess = Packet::default();
        mess.set_cmd(RoomCode::OffLine.into_u32());
        mess.set_user_id(*user_id.unwrap());
        lock.write_to_game(mess);
    }

    ///当有连接建立并open的时候调用
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        if let Some(addr) = shake.remote_addr()? {
            debug!("Connection with {} now open", addr);
        }
        self.add = Some(shake.remote_addr().unwrap().unwrap());
        info!("Connection with {:?} now open", shake.remote_addr()?);

        Ok(())
    }

    ///接收消息的时候调用
    fn on_message(&mut self, msg: WMessage) -> Result<()> {
        info!("GateServer got message '{}'. ", msg);
        //如果是二进制数据
        if msg.is_binary() {
            // let mut packet = Packet::from_only_client(msg.into_data()).unwrap();
            // let cmd = packet.get_cmd();
            // let res = self.handle_binary(packet);
            // if res.is_err() {
            //     let str = res.err().unwrap().to_string();
            //     error!("{:?}", str.as_str());
            //     if cmd == GameCode::Login as u32 {
            //         let mut res = S_USER_LOGIN::new();
            //         res.set_is_succ(false);
            //         res.set_err_mess(str);
            //         self.write_to_client(res.write_to_bytes().unwrap());
            //     }
            // }
        } else if msg.is_text() {
            //如果是文本数据
            let res = self.ws.send("hello client!");
            if res.is_err() {
                error!("{:?}", res.err().unwrap().to_string());
            }
        }
        Ok(())
    }
    ///关闭的时候调用
    fn on_close(&mut self, _cc: CloseCode, _str: &str) {
        let res = self.ws.close(CloseCode::Normal);
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
            return;
        }
        info!(
            "客户端断开连接,通知游戏服卸载玩家数据{}",
            self.add.as_ref().unwrap()
        );
        let token = self.ws.token().0;
        let mut lock = block_on(self.cm.lock());
        let user_id = lock.get_channels_user_id(&token);
        if user_id.is_none() {
            return;
        }
        let mut packet = Packet::default();
        packet.set_user_id(*user_id.unwrap());

        let cmd = RoomCode::OffLine.into_u32();

        packet.set_cmd(cmd);
        lock.write_to_game(packet.clone());

        packet.set_cmd(cmd);
        lock.write_to_game_center(packet);
        lock.close_remove(&token);
    }

    ///发送错误的时候调用
    fn on_error(&mut self, _err: WsError) {
        let res = self.ws.close(CloseCode::Error);
        if res.is_err() {
            error!("{:?}", res.err().unwrap().to_string());
        }
    }
}

impl WebSocketHandler {
    #[warn(dead_code)]
    fn handle_binary(&mut self, mut packet: Packet) -> anyhow::Result<()> {
        let token = self.ws.token().0;
        let mut lock = block_on(self.cm.lock());
        let user_id = lock.get_channels_user_id(&token);

        //如果内存不存在数据，请求的命令又不是登录命令,则判断未登录异常操作
        if user_id.is_none() && packet.get_cmd() != GameCode::Login as u32 {
            let str = format!(
                "this player is not login!cmd:{},token:{}",
                packet.get_cmd(),
                token
            );

            anyhow::bail!(str)
        }
        //执行登录
        if packet.get_cmd() == GameCode::Login as u32 {
            let mut c_login = C_USER_LOGIN::new();
            c_login.merge_from_bytes(packet.get_data())?;

            //校验用户中心账号是否已经登陆了
            let res = check_uc_online(&0)?;
            if res {
                //校验内存
                let res = check_mem_online(&0, &mut lock);
                if !res {
                    //todo
                } else {
                    let mut res = S_USER_LOGIN::new();
                    res.set_is_succ(false);
                    res.set_err_mess("this account already login!".to_owned());
                    std::mem::drop(lock);
                    let res = self.ws.send(res.write_to_bytes().unwrap());
                    if res.is_err() {
                        error!("{:?}", res.err().unwrap().to_string());
                    }
                    let str = format!("this account already login!user_id:{}", &0);

                    anyhow::bail!("{:?}", str)
                }
            }

            //校验内存是否已经登陆了(单一校验内存是否在线先保留在这)
            //check_mem_online(&c_login.get_userId(), &mut lock);

            //添加到内存
            lock.add_gate_user(0, Some(self.ws.clone()), None);
        }
        //封装packet转发到其他服
        let user_id = lock.get_channels_user_id(&token);
        packet.set_user_id(*user_id.unwrap());
        std::mem::drop(lock);
        //转发函数
        self.arrange_packet(packet);
        Ok(())
    }

    ///数据包转发
    fn arrange_packet(&mut self, packet: Packet) {
        let mut lock = block_on(self.cm.lock());
        //转发到游戏服
        if packet.get_cmd() >= GameCode::Min as u32 && packet.get_cmd() <= GameCode::Max as u32 {
            lock.write_to_game(packet.clone());
        }
        //转发到房间服
        if packet.get_cmd() >= RoomCode::Min as u32 && packet.get_cmd() <= RoomCode::Max as u32 {
            lock.write_to_game_center(packet);
        }
    }
}
