use crate::mgr::room_mgr::RoomMgr;
use protobuf::Message;
use rand::Rng;
use tools::cmd_code::ClientCode;
use tools::protos::room::{C_SKIP_TURN_CHOICE, S_SKIP_TURN_CHOICE_NOTICE};
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

    //跳过当前这个人，直接选中下一个
    let size = room.get_choice_orders().len();
    let mut index = room.get_next_choice_index();
    if index + 1 < size - 1 {
        index += 1;
        room.set_next_choice_index(index);
        let mut sstcn = S_SKIP_TURN_CHOICE_NOTICE::new();
        sstcn.set_user_id(user_id);
        let bytes = sstcn.write_to_bytes().unwrap();
        for i in room.member_index.to_vec() {
            room.send_2_client(ClientCode::SkipTurnNotice, i, bytes.clone());
        }
    } else if index + 1 == size - 1 {
        //先选出可以随机的下标
        let mut index_v: Vec<usize> = Vec::new();
        for index in 0..room.get_turn_orders().len() {
            if room.get_turn_orders()[index] != 0 {
                continue;
            }
            index_v.push(index);
        }
        let mut rand = rand::thread_rng();
        //如果是最后一个，直接给所有未选的玩家进行随机
        for member_id in &room.member_index[..] {
            //选过了就跳过
            if room.get_turn_orders().contains(member_id) {
                continue;
            }
            //没选过就系统随机
            room.choice_turn(*member_id, None);
        }
    }

    //判断当前玩家是不是最后一个，如果是，则直接给所有未选中的玩家进行随机

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
