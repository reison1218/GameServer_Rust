use std::collections::HashMap;

use super::RankInfo;

///排行榜管理器
#[derive(Default)]
pub struct RankMgr<'a>{
    rank_data:Vec<String>,//排行榜数据
    update_map:HashMap<u32,& 'a mut String>,
}

impl RankMgr <'_>{
    ///更新排行榜
    pub fn update_rank_info(&mut self){
        let self_ptr = self as *mut RankMgr;
        unsafe {
            let self1 = self_ptr.as_mut().unwrap();
            let self2 = self_ptr.as_mut().unwrap();
            self1.rank_data.push("value".to_owned());
            let res = self1.rank_data.get_mut(0).unwrap();
            self2.update_map.insert(0, res);
            let res = self2.update_map.get_mut(&0).unwrap();
            res.push_str("sdfsf");
            let res=self2.rank_data.get_mut(0).unwrap();
            println!("{:?}",res);
        }
    }
}




