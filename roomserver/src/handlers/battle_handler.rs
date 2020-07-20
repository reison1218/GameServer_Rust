use crate::entity::battle::ActionType;
use crate::mgr::room_mgr::RoomMgr;
use log::{error, info, warn};
use protobuf::Message;
use tools::protos::battle::C_ACTION;
use tools::util::packet::Packet;

///翻地图块
pub fn action(rm: &mut RoomMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let res = rm.get_room_mut(&user_id);
    if let None = res {
        return Ok(());
    }
    let room = res.unwrap();
    if room.get_next_choice_user() != user_id {
        return Ok(());
    }

    let mut ca = C_ACTION::new();
    let res = ca.merge_from_bytes(packet.get_data());
    if res.is_err() {
        error!("{:?}", res.err().unwrap());
        return Ok(());
    }
    let action_type = ca.get_action_type();

    let action_type = ActionType::from(action_type);
    match action_type {
        ActionType::None => {}
        ActionType::UseItem => {}
        ActionType::Skip => {
            skip_choice_turn(rm, user_id);
        }
        ActionType::Open => {}
        ActionType::Skill => {}
        ActionType::Attack => {}
    }
    Ok(())
}

///跳过选择回合顺序
fn skip_choice_turn(rm: &mut RoomMgr, user_id: u32) -> anyhow::Result<()> {
    let room = rm.get_room_mut(&user_id).unwrap();
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
