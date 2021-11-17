pub mod battle_tcp_server;
pub mod gate_tcp_server;
pub mod http;
pub mod rank_tcp_client;
pub mod room_tcp_client;

use std::collections::VecDeque;

use crate::Lock;
use async_trait::async_trait;
use log::warn;
use tools::cmd_code::{BattleCode, ClientCode, GameCode, RankCode, RoomCode};
use tools::net_message_io::TransportWay;
use tools::util::packet::Packet;

use self::battle_tcp_server::BattleTcpServerHandler;
use self::gate_tcp_server::GateTcpServerHandler;

#[async_trait]
trait Forward {
    fn get_battle_token(&self) -> Option<usize>;

    fn get_gate_token(&self) -> Option<usize>;

    fn get_game_center_mut(&mut self) -> &mut Lock;

    ///数据包转发
    async fn forward_packet(&mut self, packet_array: VecDeque<Packet>) {
        let gate_token = self.get_gate_token();
        let battle_token = self.get_battle_token();
        let lock = self.get_game_center_mut();
        let mut lock = lock.lock().await;
        for mut packet in packet_array {
            let cmd = packet.get_cmd();
            let user_id = packet.get_user_id();
            let mut bytes = packet.build_server_bytes();
            let is_broad = packet.is_broad();
            //需要自己处理的数据
            lock.handler(&packet, gate_token);
            let bytes_slice = bytes.as_slice();
            //处理公共的命令
            if cmd > ClientCode::Min.into_u32() && cmd < ClientCode::Max.into_u32() {
                //发送给客户端
                if is_broad {
                    for client in lock.gate_clients.values() {
                        client.send(bytes_slice);
                    }
                } else {
                    let res = lock.get_gate_client(user_id);
                    match res {
                        Ok(gc) => gc.send(bytes_slice),
                        Err(e) => warn!("{:?},cmd:{}", e, cmd),
                    }
                }
            } else if cmd > RoomCode::Min.into_u32()//转发给房间服
                && cmd < RoomCode::Max.into_u32()
            {
                if let Some(gate_token) = gate_token {
                    packet.set_server_token(gate_token as u32);
                    bytes = packet.build_server_bytes();
                }
                //发消息到房间服
                let res = lock.get_room_center_mut().send(bytes);
                if let Err(e) = res {
                    warn!("{:?}", e);
                }
            } else if cmd > GameCode::Min.into_u32()//转发给游戏服
                && cmd < GameCode::Max.into_u32()
            {
                //指定发给固定某个游戏服
                let server_token = packet.get_server_token() as usize;
                if server_token > 0 {
                    let gate_client = lock.gate_clients.get_mut(&server_token);
                    match gate_client {
                        Some(gate_client) => {
                            gate_client.send(bytes_slice);
                        }
                        None => {
                            warn!("could not find gate client by token:{}!", server_token);
                        }
                    }
                } else if is_broad {
                    //推送给所有游戏服
                    for gate_client in lock.gate_clients.values() {
                        gate_client.send(bytes_slice);
                    }
                } else {
                    //发给玩家所在游戏服
                    let res = lock.get_gate_client(user_id);
                    match res {
                        Ok(gc) => gc.send(bytes_slice),
                        Err(e) => warn!("{:?},cmd:{}", e, cmd),
                    }
                }
            } else if cmd > BattleCode::Min.into_u32()//转发给战斗服
                && cmd < BattleCode::Max.into_u32()
            {
                if is_broad {
                    for battle_client in lock.battle_clients.values() {
                        battle_client.send(bytes_slice);
                    }
                } else {
                    let res = lock.get_battle_client(user_id);
                    match res {
                        Ok(gc) => gc.send(bytes_slice),
                        Err(e) => warn!("{:?},cmd:{:?}", e, cmd),
                    }
                }
            } else if cmd > RankCode::Min.into_u32()//转发给排行榜服
                && cmd < RankCode::Max.into_u32()
            {
                if let Some(gate_token) = gate_token {
                    packet.set_server_token(gate_token as u32);
                    bytes = packet.build_server_bytes();
                }

                let rs = lock.get_rank_center_mut();
                let res = rs.send(bytes);
                match res {
                    Ok(_) => {}
                    Err(e) => warn!("{:?},cmd:{:?}", e, cmd),
                }
            } else {
                warn!("could not find cmd {}!", cmd);
                return;
            }
            //玩家离开房间，解除玩家绑定
            lock.user_leave(cmd, user_id);
            //战斗结束处理复制均衡资源回收
            lock.slb_back(cmd, battle_token);
        }
    }
}

async fn new_battle_server_tcp(address: String, handler: BattleTcpServerHandler) {
    tools::net_message_io::run(TransportWay::Tcp, address.as_str(), handler);
}

async fn new_gate_server_tcp(address: String, handler: GateTcpServerHandler) {
    tools::net_message_io::run(TransportWay::Tcp, address.as_str(), handler);
}
