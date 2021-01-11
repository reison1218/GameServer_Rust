use std::collections::HashMap;

use super::RankInfo;

///排行榜管理器
pub struct RankMgr{
    rank_data:Vec<String>,//排行榜数据
    update_map:HashMap<u32,&'static String>,
}

impl RankMgr {
    ///更新排行榜
    pub fn update_rank_info(&'static mut self,mut rank_info:RankInfo){
        self.rank_data.push("value".to_owned());
        let res = self.rank_data.get_mut(0).unwrap();
        self.update_map.insert(0, res);
        let res = self.update_map.get_mut(&0).unwrap();
        res.push_str("sdfsf");
        self.rank_data.get(0).unwrap();
    }
}




