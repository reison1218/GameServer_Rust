use crate::mgr::room_mgr::RoomMgr;
use tools::util::packet::Packet;

///跳过选择回合顺序
pub fn skip_choice_turn(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);
    if let None = res {
        return Ok(());
    }
    let room = res.unwrap();
    if room.get_next_choice_user() != user_id {
        return Ok(());
    }

    //判断是否是轮到自己操作
    let index = room.get_next_choice_index();
    let next_user = room.get_choice_orders()[index];
    if next_user != user_id {
        return Ok(());
    }
    //跳过当前这个人
    room.skip_choice_turn(user_id);
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
