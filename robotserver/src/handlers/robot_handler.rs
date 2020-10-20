use crate::mgr::robot_mgr::{GetMutRef, RobotMgr};
use protobuf::Message;
use std::collections::HashSet;
use tools::protos::robot::{C_REQUEST_ROBOT, S_REQUEST_ROBOT};
use tools::util::packet::Packet;

///请求机器人
pub fn request_robot(rm: &RobotMgr, packet: Packet) -> anyhow::Result<()> {
    let mut crr = C_REQUEST_ROBOT::default();
    let room_id = crr.get_room_id();
    let need_num = crr.need_num;
    let tile_map = crr.tile_map.clone();
    let already_cters = crr.already_cter.clone();
    let rm = rm.get_mut_ref();
    //如果没有这个房间的机器人，则初始化机器人
    if !rm.room_robot_map.contains_key(&room_id) {
        rm.room_robot_map.insert(room_id, HashSet::new());
    }

    //如果机器人不够了，就添加机器人
    if need_num > rm.idle_robot_num {
        rm.add_robot();
    }
    //将机器人添加到房间
    rm.add_robot_to_room(room_id, need_num, already_cters);

    srr.robots.Ok(())
}
