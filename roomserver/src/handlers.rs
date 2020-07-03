pub mod room_handler;
use crate::entity::member::{Member, MemberState};
use crate::entity::room::{RoomMemberNoticeType, RoomState};
use crate::entity::room_model::{BattleType, RoomModel, RoomType, TeamId};
use crate::mgr::room_mgr::RoomMgr;
use log::{error, info, warn};
use protobuf::Message;
use tools::cmd_code::{ClientCode, RoomCode};
use tools::protos::room::{
    C_CHANGE_TEAM, C_CHOOSE_CHARACTER, C_EMOJI, C_KICK_MEMBER, C_PREPARE_CANCEL, C_ROOM_SETTING,
    S_LEAVE_ROOM, S_ROOM_SETTING,
};
use tools::protos::server_protocol::{G_R_CREATE_ROOM, G_R_JOIN_ROOM, G_R_SEARCH_ROOM};
use tools::templates::emoji_temp::EmojiTemp;
use tools::util::packet::Packet;
