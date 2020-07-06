use crate::mgr::room_mgr::RoomMgr;
use tools::util::packet::Packet;

///改变目标
pub fn change_target(_rm: &mut RoomMgr, _packet: Packet) -> anyhow::Result<()> {
    Ok(())
}
