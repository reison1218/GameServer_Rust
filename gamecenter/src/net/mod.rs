use crate::mgr::game_center_mgr::GameCenterMgr;
use async_std::sync::Mutex;
use std::sync::Arc;
use tools::util::packet::Packet;

pub mod battle_tcp_server;
pub mod gate_tcp_server;

///处理客户端消息
async fn handler_mess_s(rm: Arc<Mutex<GameCenterMgr>>, packet: Packet) {
    let user_id = packet.get_user_id();

    let mut lock = rm.lock().await;
    lock.invok(packet);
}
