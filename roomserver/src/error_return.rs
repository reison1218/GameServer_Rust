use log::error;
use protobuf::Message;
use tools::cmd_code::ClientCode;
use tools::protos::protocol::{S_MODIFY_NICK_NAME, S_SYNC_DATA, S_USER_LOGIN};
use tools::protos::room::{
    S_CHOOSE_CHARACTER, S_CHOOSE_INDEX, S_CHOOSE_SKILL, S_CHOOSE_TURN_ORDER, S_EMOJI,
    S_KICK_MEMBER, S_LEAVE_ROOM, S_PREPARE_CANCEL, S_ROOM, S_ROOM_SETTING, S_START,
};
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

pub fn err_back(cmd: ClientCode, user_id: u32, error_mess: String, sender: &mut TcpSender) {
    let mut proto_res = Vec::new();
    match cmd {
        ClientCode::Login => {
            let mut sul = S_USER_LOGIN::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        ClientCode::SyncData => {
            let mut sul = S_SYNC_DATA::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        ClientCode::NickNameModify => {
            let mut sul = S_MODIFY_NICK_NAME::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        ClientCode::Room => {
            let mut sul = S_ROOM::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        ClientCode::LeaveRoom => {
            let mut sul = S_LEAVE_ROOM::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        ClientCode::Start => {
            let mut sul = S_START::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        ClientCode::PrepareCancel => {
            let mut sul = S_PREPARE_CANCEL::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        ClientCode::ChoiceSkill => {
            let mut sul = S_CHOOSE_SKILL::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        ClientCode::RoomSetting => {
            let mut sul = S_ROOM_SETTING::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        ClientCode::KickMember => {
            let mut sul = S_KICK_MEMBER::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        ClientCode::ChoiceCharacter => {
            let mut sul = S_CHOOSE_CHARACTER::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        ClientCode::Emoji => {
            let mut sul = S_EMOJI::new();
            sul.err_mess = error_mess;
            sul.is_succ = false;
            let res = sul.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        //选择位置返回
        ClientCode::ChoiceIndex => {
            let mut scl = S_CHOOSE_INDEX::new();
            scl.is_succ = false;
            scl.err_mess = error_mess;
            let res = scl.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        //选择回合顺序返回
        ClientCode::ChoiceRoundOrder => {
            let mut scl = S_CHOOSE_TURN_ORDER::new();
            scl.is_succ = false;
            scl.err_mess = error_mess;
            let res = scl.write_to_bytes();
            if let Err(e) = res {
                error!("{:?}", e);
                return;
            }
            proto_res = res.unwrap();
        }
        _ => {}
    }
    let bytes = Packet::build_packet_bytes(cmd as u32, user_id, proto_res, true, true);
    sender.write(bytes);
}
