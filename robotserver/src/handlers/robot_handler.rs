use crate::mgr::robot_mgr::RobotMgr;
use tools::protos::robot::C_REQUEST_ROBOT;
use tools::util::packet::Packet;

///请求机器人
pub fn request_robot(rm: &mut RobotMgr, packet: Packet) -> anyhow::Result<()> {
    let mut crr = C_REQUEST_ROBOT::new();
    crr.merge_from_bytes(packet.get_data());
    let room_id = crr.get_room_id();
    let already_cter = crr.already_cter;
    let need_index = crr.get_need_index();
    //如果没有这个房间的机器人，则初始化机器人
    if !rm.robot_map.contains_key(&room_id) {}
    Ok(())
}
