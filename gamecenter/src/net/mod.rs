pub mod battle_tcp_server;
pub mod gate_tcp_server;
pub mod room_tcp_client;
pub mod http;
pub mod rank_tcp_client;

use crate::mgr::game_center_mgr::GameCenterMgr;
use async_std::sync::Mutex;
use async_trait::async_trait;
use log::{warn,error};
use std::sync::Arc;
use tools::cmd_code::{BattleCode, ClientCode, GameCode, RankCode, RoomCode};
use tools::util::packet::Packet;
use async_std::task::block_on;
use tools::tcp::Handler;

#[async_trait]
trait Forward {

    fn get_battle_token(&self) ->Option<usize>;

    fn get_gate_token(&self) ->Option<usize>;

    fn get_game_center_mut(&mut self) -> &mut Arc<Mutex<GameCenterMgr>>;

    ///数据包转发
    async fn forward_packet(&mut self, packet_array: Vec<Packet>) {
        let gate_token = self.get_gate_token();

        let lock = self.get_game_center_mut();
        let mut lock = lock.lock().await;
        for packet in packet_array {
            let cmd = packet.get_cmd();
            let user_id = packet.get_user_id();
            let bytes = packet.build_server_bytes();

            //需要自己处理的数据
            lock.handler(&packet,gate_token);

            //处理公共的命令
            if cmd > ClientCode::Min.into_u32() && cmd < ClientCode::Max.into_u32() {
                //发送给客户端
                let res = lock.get_gate_client_mut(user_id);
                match res {
                    Ok(gc) => gc.send(bytes),
                    Err(e) => warn!("{:?},cmd:{}", e,cmd),
                }
            } else if cmd > RoomCode::Min.into_u32()//转发给房间服
                && cmd < RoomCode::Max.into_u32()
            {
                //发消息到房间服
                let res = lock.get_room_center_mut().send(bytes);
                if let Err(e) = res {
                    warn!("{:?}", e);
                }
            } else if cmd > GameCode::Min.into_u32()//转发给游戏服
                && cmd < GameCode::Max.into_u32()
            {
                if packet.is_broad(){
                    for gate_client in lock.gate_clients.values_mut(){
                        gate_client.send(bytes.clone());
                    }
                }else{
                    let res = lock.get_gate_client_mut(user_id);
                    match res {
                        Ok(gc) => gc.send(bytes),
                        Err(e) => warn!("{:?},cmd:{}", e,cmd),
                    }
                }
                
            } else if cmd > BattleCode::Min.into_u32()//转发给战斗服
                && cmd < BattleCode::Max.into_u32()
            {
                let res = lock.get_battle_client_mut(user_id);
                match res {
                    Ok(gc) => gc.send(bytes),
                    Err(e) => warn!("{:?},cmd:{:?}", e,cmd),
                }
            }else if cmd > RankCode::Min.into_u32()//转发给排行榜服
                && cmd < RankCode::Max.into_u32()
            {
                let res = lock.get_battle_client_mut(user_id);
                match res {
                    Ok(gc) => gc.send(bytes),
                    Err(e) => warn!("{:?},cmd:{:?}", e,cmd),
                }
            }else{
                warn!("could not find cmd {}!", cmd);
                return;
            }
            //玩家离开房间，解除玩家绑定
            lock.user_leave(cmd, user_id);
        }
    }
}


async fn new_server_tcp(address:String,handler:impl Handler){
    let res = block_on(tools::tcp::tcp_server::new(address, handler));
    if let Err(e) = res {
        error!("{:?}", e);
        std::process::abort();
    }
}