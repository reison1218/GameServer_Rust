pub mod battle_tcp_server;
pub mod gate_tcp_server;
pub mod room_tcp_client;

use crate::mgr::game_center_mgr::GameCenterMgr;
use async_std::sync::Mutex;
use async_trait::async_trait;
use log::warn;
use std::sync::Arc;
use tools::cmd_code::ServerCommonCode;
use tools::cmd_code::{BattleCode, ClientCode, GameCode, RoomCode};
use tools::util::packet::Packet;

#[async_trait]
trait Forward {
    fn get_game_center_mut(&mut self) -> &mut Arc<Mutex<GameCenterMgr>>;

    ///数据包转发
    async fn forward_packet(&mut self, packet_array: Vec<Packet>) {
        let lock = self.get_game_center_mut();
        let mut lock = lock.lock().await;
        for packet in packet_array {
            let cmd = packet.get_cmd();
            let user_id = packet.get_user_id();
            let bytes = packet.build_server_bytes();

            //需要自己处理的数据
            lock.handler(&packet);

            //处理公共的命令
            if cmd >= ServerCommonCode::LineOff.into_u32()
                && cmd <= ServerCommonCode::UpdateSeason.into_u32()
            {
                //发给房间中心
                let res = lock.get_room_center_mut().send(bytes.clone());
                if let Err(e) = res {
                    warn!("{:?}", e);
                }

                //发给战斗服
                let res = lock.get_battle_client_mut(user_id);
                match res {
                    Ok(gc) => gc.send(bytes),
                    Err(e) => warn!("{:?}", e),
                }
            } else if cmd > ClientCode::Min.into_u32() && cmd < ClientCode::Max.into_u32() {
                //发送给客户端
                let res = lock.get_gate_client_mut(user_id);
                match res {
                    Ok(gc) => gc.send(bytes),
                    Err(e) => warn!("{:?}", e),
                }
            } else if cmd > RoomCode::Min.into_u32()//转发给房间服
                && cmd < RoomCode::Max.into_u32()
            {
                let res = lock.get_room_center_mut().send(bytes);
                if let Err(e) = res {
                    warn!("{:?}", e);
                }
            } else if cmd > GameCode::Min.into_u32()//转发给游戏服
                && cmd < GameCode::Max.into_u32()
            {
                let res = lock.get_gate_client_mut(user_id);
                match res {
                    Ok(gc) => gc.send(bytes),
                    Err(e) => warn!("{:?}", e),
                }
            } else if cmd > BattleCode::Min.into_u32()//转发给战斗服
                && cmd < BattleCode::Max.into_u32()
            {
                let res = lock.get_battle_client_mut(user_id);
                match res {
                    Ok(gc) => gc.send(bytes),
                    Err(e) => warn!("{:?}", e),
                }
            }

            //玩家离开房间，解除玩家绑定
            lock.user_leave(cmd, user_id);
        }
    }
}
