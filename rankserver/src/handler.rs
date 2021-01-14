use crate::mgr::RankInfo;
use crate::mgr::{rank_mgr::RankMgr, RankInfoPtr};
use protobuf::Message;
use tools::protos::base::SummaryDataPt;
use tools::util::packet::Packet;

///更新赛季
pub fn update_rank(rm: &mut RankMgr, packet: Packet) -> anyhow::Result<()> {
    let mut sd = SummaryDataPt::new();
    sd.merge_from_bytes(packet.get_data());
    let ri = RankInfo::new(1, "name".to_owned());
    let ri_res = rm.update_map.get_mut(&1);
    if let None = ri_res {
        rm.rank_vec.push(ri);
        let len = rm.rank_vec.len();
        let res = rm.rank_vec.get_mut(len - 1).unwrap();
        rm.update_map.insert(1, RankInfoPtr(res as *mut RankInfo));
    } else {
    }
    Ok(())
}
