use super::*;
use crate::ID;
use log::kv::ToValue;
use protobuf::ProtobufEnum;
use redis::{Commands, FromRedisValue};
use serde::Deserializer;
use serde_json::{json, Map, Value};
use std::str::FromStr;
use std::sync::atomic::Ordering;
use std::sync::RwLockWriteGuard;
use tools::cmd_code::GameCode::LineOff;
use tools::cmd_code::RoomCode;
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
        info!("GateServer got message '{:?}'. ", mess);
        let mut packet = Packet::from_only_client(mess);
        match packet {
            Ok(p) => {
                self.handle_binary(p);
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
        let mut write = self.cm.write().unwrap();
        let user_id = write.get_channels_user_id(&token);

        //如果内存不存在数据，请求的命令又不是登录命令,则判断未登录异常操作
        if user_id.is_none() && packet.get_cmd() != GameCode::Login as u32 {
            error!(
                "this player is not login!cmd:{},token:{}",
                packet.get_cmd(),
                token
            );
            return;
        }
        //执行登录
        if packet.get_cmd() == GameCode::Login as u32 {
            let mut c_login = C_USER_LOGIN::new();
            let result = c_login.merge_from_bytes(packet.get_data());
            if result.is_err() {
                error!("protobuf转换错误：{:?}", result.err().unwrap());
                return;
            }

            //校验用户中心账号是否已经登陆了
            let mut res = check_uc_online(&c_login.get_user_id(), &mut write);
            if res {
                //校验内存
                res = check_mem_online(&c_login.get_user_id(), &mut write);
                if !res {
                    modify_redis_user(c_login.get_user_id(), false);
                } else {
                    let mut res = S_USER_LOGIN::new();
                    res.set_is_succ(false);
                    res.set_err_mess("this account already login!".to_owned());
                    std::mem::drop(write);
                    self.write_to_client(res.write_to_bytes().unwrap());
                    error!(
                        "this account already login!user_id:{}",
                        &c_login.get_user_id()
                    );
                    return;
                }
            }

            //校验内存是否已经登陆了(单一校验内存是否在线先保留在这)
            //check_mem_online(&c_login.get_userId(), &mut write);

            //添加到内存
            write.add_gate_user(
                c_login.get_user_id(),
                None,
                Some(self.tcp.as_ref().unwrap().clone()),
            );
        }

        //封装packet转发到其他服
        let user_id = write.get_channels_user_id(&token);
        packet.set_user_id(*user_id.unwrap());
        //释放write指针，绕过编译器检查
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
    tools::tcp::tcp_server::new(address, sh);
}
