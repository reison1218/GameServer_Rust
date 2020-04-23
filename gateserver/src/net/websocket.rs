use super::*;
use crate::entity::gateuser::GateUser;
use crate::ID;
use protobuf::ProtobufEnum;
use std::borrow::BorrowMut;
use std::io::Write;
use std::net::TcpStream;
use std::process::id;
use tools::cmd_code::{RoomCode, ClientCode};
use std::error::Error;

pub struct ClientSender {
    pub user_id: Option<u32>,
    ws: WsSender,
}

///websockethandler
/// 监听websocket网络事件
pub struct WebSocketHandler {
    pub ws: Arc<WsSender>,           //相当于channel
    pub add: Option<String>,         //客户端地址
    pub cm: Arc<RwLock<ChannelMgr>>, //channel管理结构体指针
}

///实现相应的handler函数
impl Handler for WebSocketHandler {
    //关闭的时候调用
    fn on_shutdown(&mut self) {
        self.ws.close(CloseCode::Invalid);
        let token = self.ws.token().0;
        let mut write = self.cm.write().unwrap();
        write.close_remove(&token);
        let user_id = write.get_channels_user_id(&token);
        let mut mess = MessPacketPt::new();
        mess.set_cmd(tools::cmd_code::GameCode::LineOff as u32 as u32);
        mess.set_user_id(*user_id.unwrap());
        write.write_to_game(mess);
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
            let bytes = &msg.into_data()[..];
            self.handle_binary(bytes);
        } else if msg.is_text() {
            //此处代码做性能测试
            //以下代码做性能测试
            let mut mp = MessPacketPt::new();
            let mut s_l = C_USER_LOGIN::new();
            s_l.set_avatar("test".to_owned());
            s_l.set_nickName("test".to_owned());
            {
                ID.write().unwrap().id += 1;
            }
            let id = ID.write().unwrap().id;

            s_l.set_userId(id);
            mp.set_user_id(0);
            mp.set_cmd(GameCode::Login as u32);
            mp.set_is_broad(false);
            mp.set_is_client(true);
            let result = s_l.write_to_bytes();
            if result.is_err() {
                error!("protobuf转换错误：{:?}", result.err().as_mut().unwrap());
                return Ok(());
            }
            mp.set_data(result.unwrap());
            self.handle_binary(mp.get_data());
            //如果是文本数据
            self.ws.send("hello client!");
        }
        Ok(())
    }
    ///关闭的时候调用
    fn on_close(&mut self, cc: CloseCode, _str: &str) {
        self.ws.close(CloseCode::Normal);
        info!(
            "客户端断开连接,通知游戏服卸载玩家数据{}",
            self.add.as_ref().unwrap()
        );
        let token = self.ws.token().0;
        let mut write = self.cm.write().unwrap();
        let user_id = write.get_channels_user_id(&token);
        if user_id.is_none() {
            return;
        }
        let mut mess = MessPacketPt::new();
        mess.set_cmd(tools::cmd_code::GameCode::LineOff as u32);
        mess.set_user_id(*user_id.unwrap());
        write.write_to_game(mess);
        write.close_remove(&token);
    }

    ///发送错误的时候调用
    fn on_error(&mut self, err: WsError) {
        self.ws.close(CloseCode::Error);
    }
}

impl WebSocketHandler {
    fn handle_binary(&mut self, bytes: &[u8]) {
        let mut mp = bytes_to_mess_packet_pt(bytes);

        let token = self.ws.token().0;
        let mut write = self.cm.write().unwrap();
        let user_id = write.get_channels_user_id(&token);

        //如果内存不存在数据，请求的命令又不是登录命令,则判断未登录异常操作
        if user_id.is_none() && mp.get_cmd() != GameCode::Login as u32 {
            error!(
                "this player is not login!cmd:{},token:{}",
                mp.get_cmd(),
                token
            );
            return;
        }
        //执行登录
        if mp.get_cmd() == GameCode::Login as u32 {
            let mut gate_user = write.get_mut_user_channel_channel(&mp.get_user_id());
            //如果有，则执行T下线
            if gate_user.is_some() {
                let token = gate_user.as_mut().unwrap().get_ws_mut_ref().token().0;
                //释放可变指针，免得出现重复可变指针编译不通过
                std::mem::drop(gate_user.unwrap());
                write.close_remove(&token);
            }
            write.add_gate_user(mp.get_user_id(), Some(self.ws.clone()), None);
        }
        std::mem::drop(write);
        //转发函数
        self.arrange_packet(mp);
    }

    ///数据包转发
    fn arrange_packet(&mut self, packet: MessPacketPt) {
        //转发到游戏服
        if packet.get_cmd() >= GameCode::Min as u32&& packet.get_cmd() <= GameCode::Max as u32{
            let mut write = self.cm.write().unwrap();
            write.write_to_game(packet);
            return;
        }
        //转发到房间服
        if packet.get_cmd() >= RoomCode::Min as u32 && packet.get_cmd() <= RoomCode::Max as u32 {
            return;
        }
    }
}

///byte数组转换Packet
pub fn build_packet(mess: MessPacketPt) -> Packet {
    //封装成packet
    let mut packet = Packet::new(mess.cmd);
    packet.set_data(&mess.write_to_bytes().unwrap()[..]);
    packet
}
