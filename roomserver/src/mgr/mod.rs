pub mod room_mgr;
use serde::{Deserialize, Serialize};
use tools::protos::base::LeaguePt;

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

impl From<&LeaguePt> for League {
    fn from(lp: &LeaguePt) -> Self {
        let mut l = League::default();
        l.league_score = lp.get_league_score();
        l.league_time = lp.league_time;
        let league_id = lp.league_id as i8;
        l.id = league_id;
        l
    }
}

impl League {
    pub fn into_pt(&self) -> LeaguePt {
        let mut lpt = LeaguePt::new();
        lpt.set_league_id(self.id as i32);
        lpt.set_league_score(self.league_score);
        lpt.set_league_time(self.league_time);
        lpt
    }

    pub fn update(&mut self, league_id: i8, league_score: i32, league_time: i64) {
        self.id = league_id;
        self.league_score = league_score;
        self.league_time = league_time;
    }
}
