use super::*;
use crate::entity::dao;
use crate::entity::user::User;
use crate::net::channel::Channel;
use crate::protos::base::loginRsp;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use std::rc::Rc;

///websockethandler
/// 监听websocket网络事件
pub struct WebSocketHandler {
    pub ws: Arc<WsSender>,   //相当于channel
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
            let array = &msg.into_data()[..];
            //封装到proto里面去
            test.merge_from_bytes(array);
            println!("{}", test.get_cmd());
            //写回去
            test.set_cmd(939434);

            self.ws.send(&test.write_to_bytes().unwrap()[..]);

            //封装成packet
            let mut t = Test::new();

            t.merge_from_bytes(array);
            let mut pd = PacketDes::new(t.cmd, t.userId);
            let mut packet = Packet::new(pd);
            //判断是否执行登录
            if t.cmd == 1 {
                self.login(packet);
            } else {
                let mut gm = self.gm.lock().unwrap();
                gm.invok(packet);
            }
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
        close(self);
    }
    //关闭的时候调用
    fn on_shutdown(&mut self) {
        self.ws.close(CloseCode::Invalid);
        close(self);
    }

    ///发送错误的时候调用
    fn on_error(&mut self, err: Error) {
        self.ws.close(CloseCode::Error);
        close(self);
    }
}

///当websocket断开时候调用
fn close(handler: &mut WebSocketHandler) {
    let token = handler.ws.token().0;
    let mut gm = handler.gm.lock().unwrap();
    //调用离线函数
    gm.logOff(&token);
    println!("客户端断开连接,{}", handler.add.as_ref().unwrap());
}

impl WebSocketHandler {
    //登录函数，执行登录
    fn login(&mut self, packet: Packet) {
        let mut gm = self.gm.lock().unwrap();

        //校验是否已经登录
        let user_id = packet.packet_des.user_id;
        if gm.players.contains_key(&user_id) {
            return;
        }

        //走登录流程
        let mut user = User::query(user_id, &mut gm.pool);
        if user.is_none() {
            return;
        }
        let mut user = user.unwrap();
        let mut time = std::time::SystemTime::now().elapsed().unwrap();
        user.login_time =
            chrono::NaiveDateTime::from_timestamp(time.as_secs() as i64, time.subsec_nanos());

        //封装到内存中
        gm.players.insert(user_id.clone(), user);

        //封装会话
        let channel = Channel::init(user_id.clone(), self.ws.clone());
        let token = channel.sender.token().0;
        gm.channels.insert_user_channel(user_id.clone(), channel);
        gm.channels.insert_channels(token, user_id.clone());
        let user = gm.players.get(&user_id).unwrap();
        //返回客户端
        let mut lr = user2proto(&user);
        info!("用户完成登录！{}", &user_id);
        gm.channels
            .get_mut_user_channel(&user_id)
            .unwrap()
            .sender
            .send(&lr.write_to_bytes().unwrap()[..]);
    }
}

fn user2proto(user: &User) -> loginRsp {
    let mut lr = loginRsp::new();
    lr.userId = user.id;
    lr.account = user.account.clone();
    lr.gold = user.gold;
    lr.platform = user.platform.clone();
    lr.token = user.token.clone();
    lr.login_time = user.login_time.timestamp_subsec_nanos();
    lr.channel = user.channel.clone();
    lr
}
