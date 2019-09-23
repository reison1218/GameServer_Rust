use super::*;

///websockethandler
/// 监听websocket网络事件
pub struct WebSocketHandler {
    pub ws: WsSender,        //相当于channel
    pub add: Option<String>, //客户端地址
    pub gm: Arc<Mutex<GameMgr>>,
}

///实现相应的handler函数
impl Handler for WebSocketHandler {
    ///接收消息的时候调用
    fn on_message(&mut self, msg: WMessage) -> Result<()> {
        println!("Server got message '{}'. ", msg);

        //如果是二进制数据
        if msg.is_binary() {
            //转换成u8集合
            let mut test = Test::new();
            //封装到proto里面去
            test.merge_from_bytes(&msg.into_data()[..]);
            println!("{}", test.get_a());
            //写回去
            test.set_a(939434);

            self.ws.send(&test.write_to_bytes().unwrap()[..]);
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
