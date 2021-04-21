use std::collections::VecDeque;

use super::*;
use async_std::sync::Mutex;
use async_std::task::block_on;
use async_trait::async_trait;
use crossbeam::channel::Sender;
use log::error;
use tools::cmd_code::{ClientCode, GateCode, RankCode, RoomCode, ServerCommonCode};

pub enum TcpClientType {
    GameServer,
    GameCenter,
}

pub struct TcpClientHandler {
    client_type: TcpClientType,
    ts: Option<Sender<Vec<u8>>>,
    cp: Arc<Mutex<ChannelMgr>>,
}

impl TcpClientHandler {
    pub fn new(cp: Lock, client_type: TcpClientType) -> TcpClientHandler {
        let tch = TcpClientHandler {
            ts: None,
            cp,
            client_type,
        };
        tch
    }
}

#[async_trait]
impl ClientHandler for TcpClientHandler {
    async fn on_open(&mut self, ts: Sender<Vec<u8>>) {
        match self.client_type {
            TcpClientType::GameServer => {
                block_on(self.cp.lock()).set_game_client_channel(ts.clone());
            }
            TcpClientType::GameCenter => {
                block_on(self.cp.lock()).set_game_center_client_channel(ts.clone());
            }
        }
        self.ts = Some(ts);
    }

    async fn on_close(&mut self) {
        let address: Option<&str>;
        match self.client_type {
            TcpClientType::GameServer => {
                address = Some(CONF_MAP.get_str("game_port"));
            }
            TcpClientType::GameCenter => {
                address = Some(CONF_MAP.get_str("game_center_port"));
            }
        }
        self.on_read(address.unwrap().to_string()).await;
    }

    async fn on_message(&mut self, mess: Vec<u8>) {
        let packet_array = Packet::build_array_from_server(mess);

        if let Err(e) = packet_array {
            error!("{:?}", e.to_string());
            return;
        }
        let packet_array = packet_array.unwrap();
        handler_mess_s(self.cp.clone(), packet_array)
    }
}

fn handler_mess_s(cp: Lock, packet_array: VecDeque<Packet>) {
    for mut packet in packet_array {
        let mut lock = async_std::task::block_on(cp.lock());
        let user_id = packet.get_user_id();
        let cmd = packet.get_cmd();
        //判断是否是发给客户端消息
        if packet.is_client() && cmd > 0 {
            if cmd == ClientCode::Login.into_u32() {
                //封装成gateuser到管理器中
                lock.temp_channel_2_gate_user(user_id);
            }
            let gate_user = lock.get_mut_user_channel_channel(&user_id);
            match gate_user {
                Some(user) => {
                    user.get_tcp_mut_ref().send(packet.build_client_bytes());
                    info!("回给客户端消息,user_id:{},cmd:{}", user_id, cmd,);
                }
                None => {
                    if cmd == ClientCode::LeaveRoom.into_u32()
                        || cmd == ClientCode::MemberLeaveNotice.into_u32()
                    {
                        continue;
                    }
                    warn!("user data is null,id:{},cmd:{}", &user_id, cmd);
                }
            }
        } else {
            //判断是否要转发到其他服务器进程消息
            arrange_packet(lock, packet);
        }
    }
}

///数据包转发
fn arrange_packet(cp: async_std::sync::MutexGuard<ChannelMgr>, packet: Packet) {
    let cmd = packet.get_cmd();
    let mut lock = cp;
    //转发到游戏服
    if cmd == ServerCommonCode::ReloadTemps.into_u32()
        || (cmd >= GameCode::Min.into_u32() && cmd <= GameCode::Max.into_u32())
    {
        if cmd == GameCode::UnloadUser.into_u32() {
            let user_id = packet.get_user_id();
            let gate = lock.get_user_channel(&user_id);
            if let Some(gate) = gate {
                let token = gate.get_token();
                //关闭连接
                lock.close_remove(&token);
            }
        }
        lock.write_to_game(packet);
    } else if (cmd >= RoomCode::Min.into_u32() && cmd <= RoomCode::Max.into_u32())
        || (cmd >= RankCode::Min.into_u32() && cmd <= RankCode::Max.into_u32())
    {
        //转发到房间服
        lock.write_to_game_center(packet);
    } else if cmd == GateCode::StopServer.into_u32() {
        lock.stop_server();
    }
}
