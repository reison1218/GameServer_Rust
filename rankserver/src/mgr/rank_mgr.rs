use std::collections::HashMap;

use super::RankInfo;

///排行榜管理器
#[derive(Default)]
pub struct RankMgr<'a>{
    rank_data:Vec<RankInfo>,//排行榜数据
    update_map:HashMap<u32,&'a mut RankInfo>,
}

impl RankMgr <'_>{
    ///更新排行榜
    pub fn update_rank_info(&mut self){
        let self_ptr = self as *mut RankMgr;
        unsafe {
            let self1 = self_ptr.as_mut().unwrap();
            let self2 = self_ptr.as_mut().unwrap();

        }
    }
}




