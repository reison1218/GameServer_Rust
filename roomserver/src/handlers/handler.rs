use crate::mgr::room_mgr::RoomMgr;
use tools::util::packet::Packet;

///行动请求
#[track_caller]
pub fn do_something(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    Ok(())
}
