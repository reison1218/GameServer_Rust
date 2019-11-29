use super::*;

///websockethandler
/// 监听websocket网络事件
pub struct WebSocketHandler {
    pub ws: Arc<WsSender>,        //相当于channel
    pub add: Option<String>,      //客户端地址
    pub gm: Arc<RwLock<GameMgr>>, //gamemgr的引用计数器
}

///实现相应的handler函数
impl Handler for WebSocketHandler {
    ///接收消息的时候调用
    fn on_message(&mut self, msg: WMessage) -> Result<()> {
        println!("Server got message '{}'. ", msg);

        //处理二进制数据
        if msg.is_binary() {
            self.handle_binary(&msg.into_data()[..]);
            return Ok(());
        }

        //处理文本数据
        if msg.is_text() {
            let text = msg.into_text();
            match text {
                Ok(ok) => self.handle_text(ok),
                Err(e) => error!("data is text,but occur error!{:?}", e.to_string()),
            };
        }

        // echo it back
        Ok(())
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

    ///关闭的时候调用
    fn on_close(&mut self, _: CloseCode, _str: &str) {
        self.ws.close(CloseCode::Normal).unwrap();
        close(self);
    }
    //关闭的时候调用
    fn on_shutdown(&mut self) {
        self.ws.close(CloseCode::Invalid).unwrap();
        close(self);
    }

    ///发送错误的时候调用
    fn on_error(&mut self, err: Error) {
        self.ws.close(CloseCode::Error).unwrap();
        close(self);
    }
}

///当websocket断开时候调用
fn close(handler: &mut WebSocketHandler) {
    let token = handler.ws.token().0;
    let mut gm = handler.gm.write().unwrap();
    //调用离线函数
    gm.log_off(&token);
    println!("客户端断开连接,{}", handler.add.as_ref().unwrap());
}

impl WebSocketHandler {
    ///处理二进制
    fn handle_binary(&mut self, bytes: &[u8]) {
        //转换成u8集合
        let mut packet = build_packet(bytes);
        let mut gm = self.gm.write().unwrap();
        //判断是否执行登录
        if packet.get_cmd() == C_USER_LOGIN.value() as u32 {
            let mut c_login = C_USER_LOGIN_PROTO::new();
            let user_id = gm.channels.get_mut_channels(&self.ws.token().0);
            //已经登录过就T下线
            if user_id > 0 {
                let channel = gm.channels.get_mut_user_channel(&user_id);
                if channel.is_some() {
                    let token = &channel.unwrap().sender.token().0;
                    gm.channels.close_remove(token);
                }
            }

            c_login.merge_from_bytes(packet.get_data()).unwrap();
            packet.set_user_id(c_login.userId);
            //会话信息放入内存
            insert_channel(self.ws.clone(), gm, c_login.userId);
            self.login(packet);
        } else {
            //不登录就执行其他命令
            gm.invok(packet);
        }
    }

    ///处理文本
    fn handle_text(&mut self, str: String) {
        println!("received string from client,{:?}", str);
    }

    ///处理ping pong
    fn handle_pong(&mut self, bytes: &[u8]) {}

    //登录函数，执行登录
    fn login(&mut self, packet: Packet) {
        let mut gm = self.gm.clone();

        //玩家id
        let user_id = packet.get_user_id().unwrap();

        let mut user_data = false;
        {
            user_data = gm.read().unwrap().players.contains_key(&user_id);
        }
        //校验玩家是否登录过
        if !user_data {
            //走登录流程
            let mut m = move || {
                let mut gm = gm.write().unwrap();
                let user = User::query(user_id, &mut gm.pool);
                if user.is_none() {
                    info!("玩家数据不存在，无法登录！{}", user_id);
                    return;
                }
                let mut user = user.unwrap();
                let mut time = std::time::SystemTime::now().elapsed().unwrap();
                let mut date_time = chrono::NaiveDateTime::from_timestamp(
                    time.as_secs() as i64,
                    time.subsec_nanos(),
                );
                user.set_time("login_time".to_owned(), date_time);
                //封装到内存中
                gm.players.insert(user_id.clone(), user);

                //封装会话
                let user = gm.players.get_mut(&user_id).unwrap();
                //返回客户端
                let mut lr = user2proto(user);
                info!("用户完成登录！user_id:{}", &user_id);

                //返回客户端
                gm.channels
                    .get_mut_user_channel(&user_id)
                    .unwrap()
                    .sender
                    .send(&lr.write_to_bytes().unwrap()[..])
                    .unwrap();
            };
            &THREAD_POOL.submit_game(m);
        } else {
            //如果已有数据，直接返回客户端
            let mut gm = gm.write().unwrap();
            //封装会话
            let user = gm.players.get_mut(&user_id).unwrap();
            //返回客户端
            let mut lr = user2proto(user);
            info!("用户完成登录！user_id:{}", &user_id);

            //返回客户端
            gm.channels
                .get_mut_user_channel(&user_id)
                .unwrap()
                .sender
                .send(&lr.write_to_bytes().unwrap()[..])
                .unwrap();
        }
    }
}

///byte数组转换Packet
pub fn build_packet(bytes: &[u8]) -> Packet {
    let mut mpp = MessPacketPt::new();
    mpp.merge_from_bytes(bytes);

    //封装成packet
    let pd = PacketDes::new(mpp.cmd);
    let mut packet = Packet::new(pd);
    packet.set_bytes(&mpp.data[..]);
    packet
}

///封装会话函数
fn insert_channel(ws: Arc<WsSender>, gm: RwLockWriteGuard<GameMgr>, user_id: u32) {
    let mut gm = gm;
    let sender = ws;
    gm.channels.insert_channels(sender.token().0, user_id);
    let mut channel = Channel::init(user_id, sender);
    gm.channels.insert_user_channel(user_id, channel);
}

///user结构体转proto
fn user2proto(user: &mut User) -> S_USER_LOGIN_PROTO {
    let mut lr = S_USER_LOGIN_PROTO::new();
    lr.set_isSucc(true);
    lr.userId = user.user_id;
    lr.avatar = user.get_str(AVATAR).unwrap().to_owned();
    lr.nickName = user.get_str(NICK_NAME).unwrap().to_owned();
    let mut ppt = PlayerPt::new();
    ppt.maxcp = user.get_usize(MAX_CP).unwrap() as u32;
    ppt.maxJumpLevel = user.get_usize(MAX_JUMP_LEVEL).unwrap() as u32;
    ppt.maxMultiple = user.get_usize(MAX_MULTIPLE).unwrap() as u32;
    ppt.maxJumpRange = user.get_usize(MAX_JUMP_RANGE).unwrap() as u32;
    ppt.maxScore = user.get_usize(MAX_SCORE).unwrap() as u32;
    lr.playerPt = protobuf::SingularPtrField::some(ppt);
    lr
}

fn build_rsp_message(cmd: u32, bytes: &[u8]) {
    let mut b = bytebuf::ByteBuf::new();
    let len = bytes.bytes();
    //b.push_u32()

    let pd = PacketDes::new(cmd);
    let mut packet = Packet::new(pd);
}
