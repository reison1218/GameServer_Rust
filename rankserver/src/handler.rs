use crate::mgr::RankInfo;
use crate::mgr::{rank_mgr::RankMgr, RankInfoPtr};
use log::error;
use protobuf::Message;
use tools::protos::server_protocol::B_S_SUMMARY;
use tools::util::packet::Packet;

///更新赛季
pub fn update_rank(rm: &mut RankMgr, packet: Packet) -> anyhow::Result<()> {
    let user_id = packet.get_user_id();
    let mut bss = B_S_SUMMARY::new();
    let res = bss.merge_from_bytes(packet.get_data());
    if let Err(e) = res {
        error!("{:?}", e);
        return Ok(());
    }
    let sd = bss.get_summary_data();
    let res = rm.update_map.get_mut(&user_id);
    match res {
        Some(rank_ptr) => {
            rank_ptr.update(sd);
        }
        None => {
            let ri = RankInfo::from(sd);
            rm.rank_vec.push(ri);
            let len = rm.rank_vec.len();
            let ri_mut = rm.rank_vec.get_mut(len - 1).unwrap();
            rm.update_map
                .insert(user_id, RankInfoPtr(ri_mut as *mut RankInfo));
        }
    }
    Ok(())
}
