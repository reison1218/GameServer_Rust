use super::*;

///websockethandler
/// 监听websocket网络事件
pub struct WebSocketHandler {
    pub ws: WsSender,                //相当于channel
    pub add: Option<String>,         //客户端地址
    pub cm: Arc<RwLock<ChannelMgr>>, //channel管理器
}

///实现相应的handler函数
impl Handler for WebSocketHandler {
    ///接收消息的时候调用
    fn on_message(&mut self, msg: WMessage) -> Result<()> {
        println!("Server got message '{}'. ", msg);

        //如果是二进制数据
        if msg.is_binary() {
            let bytes = &msg.into_data()[..];
            println!("{:?}", bytes);
            self.handle_binary(bytes);
        } else if msg.is_text() {
            //如果是文本数据
            self.ws.send("hello client!");
        }
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
        self.ws.close(CloseCode::Normal);
        println!("客户端断开连接,{}", self.add.as_ref().unwrap());
    }
    //关闭的时候调用
    fn on_shutdown(&mut self) {
        self.ws.close(CloseCode::Invalid);
    }

    ///发送错误的时候调用
    fn on_error(&mut self, err: Error) {
        self.ws.close(CloseCode::Error);
    }
}

impl WebSocketHandler {
    fn handle_binary(&mut self, bytes: &[u8]) {
        let mut mess = MessPacketPt::new();
        mess.merge_from_bytes(bytes);

        let mut packet = build_packet(mess);

        self.arrange_packet(packet);
    }

    ///数据包转发
    fn arrange_packet(&mut self, packet: Packet) {
        //转发到游戏服
        if packet.get_cmd() >= GAME_MIN && packet.get_cmd() <= GAME_MAX {
            let lock = self.cm.write();
            lock.unwrap().write_to_game(packet);
            return;
        }
        //转发到房间服
        if packet.get_cmd() >= ROOM_MIN && packet.get_cmd() <= ROOM_MAX {
            return;
        }
    }
}

///byte数组转换Packet
pub fn build_packet(mess: MessPacketPt) -> Packet {
    //封装成packet
    let pd = PacketDes::new(mess.cmd);
    let mut packet = Packet::new(pd);
    packet.set_bytes(&mess.data[..]);
    packet
}
