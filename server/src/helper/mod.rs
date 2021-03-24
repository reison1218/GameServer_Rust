pub mod redis_helper;

use serde::{Deserialize, Serialize};
use tools::protos::base::{LeaguePt, RankInfoPt};

///排行榜数据结构体
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RankInfo {
    pub user_id: u32,    //玩家id
    pub name: String,    //名字
    pub rank: i32,       //排名
    pub cters: Vec<u32>, //最常用的三个角色
    pub league: League,  //段位
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct League {
    pub id: i8,            //段位id
    pub league_score: i32, //段位积分
    pub league_time: i64,  //进入段位的时间
}

impl League {
    pub fn get_league_id(&self) -> i8 {
        self.id
    }
}
impl RankInfo {
    pub fn into_rank_pt(&self) -> RankInfoPt {
        let mut rip = RankInfoPt::new();
        rip.user_id = self.user_id;
        rip.name = self.name.clone();
        rip.rank = self.rank;
        rip.set_cters(self.cters.clone());
        let mut l_pt = LeaguePt::new();
        l_pt.set_league_id(self.league.get_league_id() as i32);
        l_pt.set_league_score(self.league.league_score);
        l_pt.set_league_time(self.league.league_time);
        rip.set_league(l_pt);
        rip
    }
}
