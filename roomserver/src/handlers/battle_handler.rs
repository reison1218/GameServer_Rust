use crate::mgr::room_mgr::RoomMgr;
use tools::util::packet::Packet;

///使用道具
pub fn use_item(_rm: &mut RoomMgr, _packet: Packet) -> anyhow::Result<()> {
    Ok(())
}

///使用技能
pub fn use_skill(_rm: &mut RoomMgr, _packet: Packet) -> anyhow::Result<()> {
    Ok(())
}

///改变目标
pub fn change_target(_rm: &mut RoomMgr, _packet: Packet) -> anyhow::Result<()> {
    Ok(())
}

///攻击目标
pub fn attack_target(_rm: &mut RoomMgr, _packet: Packet) -> anyhow::Result<()> {
    Ok(())
}

///跳过turn
pub fn skip_turn(_rm: &mut RoomMgr, _packet: Packet) -> anyhow::Result<()> {
    Ok(())
}

///翻地图块
pub fn open(_rm: &mut RoomMgr, _packet: Packet) -> anyhow::Result<()> {
    
    Ok(())
}
