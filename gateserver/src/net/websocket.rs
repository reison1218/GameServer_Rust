use crate::auth::steam_auth::auth_account;
use crate::auth::STEAM;

use super::*;
use async_std::sync::{Mutex, MutexGuard};
use async_std::task::block_on;
use async_trait::async_trait;
use chrono::Local;
use tools::cmd_code::{BattleCode, ClientCode, RoomCode};
use tools::net_message_io::NetHandler;
use tools::net_message_io::TransportWay;
use tools::protos::protocol::HEART_BEAT;

#[derive(Clone)]
struct WsServerHandler {
    pub ws_handler: Option<NetHandler>, //相当于channel
    cm: Arc<Mutex<ChannelMgr>>,         //channel管理器
}

unsafe impl Send for WsServerHandler {}

unsafe impl Sync for WsServerHandler {}

#[async_trait]
impl tools::net_message_io::MessageHandler for WsServerHandler {
    async fn try_clone(&self) -> Self {
        self.clone()
    }

    async fn on_open(&mut self, tcp_handler: NetHandler) {
        self.ws_handler = Some(tcp_handler);
    }

    async fn on_close(&mut self) {
        let token = self.get_token();

        let mut lock = self.cm.lock().await;
        lock.off_line(token);
    }

    ///此处返回一个bool，表示校验数据包结果，若为false,则tcp底层将T出客户端，为true则不会
    async fn on_message(&mut self, mess: &[u8]) {
        //校验包长度
        if mess.is_empty() || mess.len() < 16 {
            error!("client packet len is wrong!");
            self.shut_down();
            return;
        }
        let packet_array = Packet::build_array_from_client(mess.to_vec());

        if packet_array.is_err() {
            error!("{:?}", packet_array.err().unwrap().to_string());
            return;
        }
        let packet_array = packet_array.unwrap();

        let mut res;
        for packet in packet_array {
            let cmd = packet.get_cmd();
            info!("GateServer receive data of client!cmd:{}", cmd);
            res = self.handle_binary(packet).await;
            if !res {
                self.shut_down();
                return;
            }
        }
    }
}

impl WsServerHandler {
    pub fn shut_down(&self) {
        let tcp = self.ws_handler.as_ref().unwrap();
        let endpoint = tcp.endpoint;
        tcp.node_handler.network().remove(endpoint.resource_id());
    }

    pub fn get_token(&self) -> usize {
        self.ws_handler
            .as_ref()
            .unwrap()
            .endpoint
            .resource_id()
            .raw()
    }

    ///写到客户端
    fn write_to_client(&mut self, bytes: &[u8]) {
        let tcp = self.ws_handler.as_ref().unwrap();
        let endpoint = tcp.endpoint;
        tcp.node_handler.network().send(endpoint, bytes);
        // let res = self.tcp.as_mut();
        // match res {
        //     Some(ts) => {
        //         ts.send(bytes);
        //     }
        //     None => {
        //         warn!("TcpServerHandler's tcp is None!");
        //     }
        // }
    }

    ///处理二进制数据
    async fn handle_binary(&mut self, mut packet: Packet) -> bool {
        let token = self.get_token();
        let mut lock = self.cm.lock().await;
        let user_id = lock.get_channels_user_id(&token);

        //如果内存不存在数据，请求的命令又不是登录命令,则判断未登录异常操作
        if user_id.is_none() && packet.get_cmd() != GameCode::Login.into_u32() {
            let str = format!(
                "this player is not login and cmd != Login!cmd:{},token:{}",
                packet.get_cmd(),
                token
            );
            warn!("{:?}", str.as_str());
            return true;
        }

        let u_id;
        //执行登录
        if packet.get_cmd() == GameCode::Login.into_u32() {
            let mut c_u_l = C_USER_LOGIN::new();
            let res = c_u_l.merge_from_bytes(packet.get_data());
            if res.is_err() {
                error!("{:?}", res.err().unwrap().to_string());
                return true;
            }
            let platform_value = c_u_l.platform_value.as_str();

            let register_platform = c_u_l.register_platform.as_str();

            let user_id = c_u_l.get_user_id();

            let res = handle_login(&mut lock, register_platform, platform_value, user_id);
            match res {
                Ok(user_id) => {
                    u_id = user_id;
                }
                Err(e) => {
                    error!("{:?}", e);
                    let mut sul = S_USER_LOGIN::new();
                    sul.set_is_succ(false);
                    sul.set_err_mess(e.to_string());
                    packet.set_cmd(ClientCode::Login as u32);
                    packet.set_data_from_vec(sul.write_to_bytes().unwrap());
                    std::mem::drop(lock);
                    self.write_to_client(packet.build_client_bytes().as_slice());
                    return false;
                }
            }
            lock.temp_channels.insert(u_id, self.ws_handler.clone());
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
                return true;
            }
            let gate_user = gate_user.unwrap();
            let tcp = gate_user.get_net_mut_ref();
            let endpoint = tcp.endpoint;
            tcp.node_handler.network().send(endpoint, res.as_slice());
            info!(
                "回给客户端消息,user_id:{},cmd:{}",
                packet.get_user_id(),
                packet.get_cmd(),
            );
        }
        std::mem::drop(lock);
        //转发函数
        self.arrange_packet(packet);
        true
    }

    ///数据包转发
    fn arrange_packet(&mut self, packet: Packet) {
        let mut lock = block_on(self.cm.lock());
        //转发到游戏服
        if packet.get_cmd() >= GameCode::Min as u32 && packet.get_cmd() <= GameCode::Max as u32 {
            lock.write_to_game(packet);
            return;
        }
        //转发到中心服
        if (packet.get_cmd() >= RoomCode::Min.into_u32()
            && packet.get_cmd() <= RoomCode::Max.into_u32())
            || (packet.get_cmd() >= BattleCode::Min.into_u32()
                && packet.get_cmd() <= BattleCode::Max.into_u32())
        {
            lock.write_to_game_center(packet);
            return;
        }
    }
}

///创建新的tcpserver并开始监听
pub fn new(address: &str, cm: Lock) {
    let sh = WsServerHandler {
        ws_handler: None,
        cm,
    };
    tools::net_message_io::run(TransportWay::Tcp, address, sh);
}

///处理登陆逻辑
fn handle_login(
    lock: &mut MutexGuard<ChannelMgr>,
    register_platform: &str,
    platform_value: &str,
    user_id: u32,
) -> anyhow::Result<u32> {
    let debug = crate::CONF_MAP.get_bool("debug", false);

    // if debug {
    //     query_user_id_from_redis(platform_value)?;
    // } else {
    //     if register_platform.eq(STEAM) {
    //         // user_id = auth_account(platform_value)?;
    //         let res = query_pid_from_redis(user_id);
    //         match res {
    //             Ok(_) => {
    //                 return Ok(user_id);
    //             }
    //             Err(e) => {
    //                 anyhow::bail!("{:?}", e)
    //             }
    //         }
    //     } else {
    //         return Ok(0);
    //     }
    // }

    let res = query_pid_from_redis(user_id);

    if let Err(e) = res {
        anyhow::bail!("{:?}", e)
    }

    // //校验内存
    let mem_res = check_mem_online(&user_id, lock);
    // //如果用户中心登陆了或者本地内存登陆了，直接错误返回
    if mem_res {
        // let str = format!("this account already login!user_id:{}", user_id);
        // warn!("{:?}", str.as_str());
        // anyhow::bail!("{:?}", str)
        lock.kick_player(user_id);
        warn!("发现重复登陆，T掉之前的tcp!user_id:{}", user_id);
    }
    Ok(user_id)
}
