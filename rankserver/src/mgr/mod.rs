use tools::protos::base::{LeaguePt, SummaryDataPt};
pub mod rank_mgr;
use serde::{Deserialize, Serialize};

pub struct RankInfoPtr(pub *mut RankInfo);

unsafe impl Send for RankInfoPtr {}

///排行榜数据结构体
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RankInfo {
    pub user_id: u32,    //玩家id
    pub name: String,    //名字
    pub rank: i32,       //排名
    pub cters: Vec<u32>, //最常用的三个角色
    pub league: League,  //段位
}

unsafe impl Send for RankInfo {}

impl RankInfo {
    ///更新段位
    pub fn update_league(&mut self, id: i8) {
        let res = crate::TEMPLATES.league_temp_mgr().get_temp(&id).unwrap();
        self.league.id = res.id;
        let time = chrono::Local::now();
        self.league.league_score = res.score;
        self.league.league_time = time.timestamp_millis();
    }

    ///获得积分
    pub fn get_score(&self) -> i32 {
        self.league.league_score
    }

    pub fn new(sd_pt: &SummaryDataPt, cters: Vec<u32>) -> Self {
        let league = League::from(sd_pt.get_league());
        RankInfo {
            user_id: sd_pt.user_id,
            name: sd_pt.name.clone(),
            rank: -1,
            cters,
            league,
        }
    }

    pub fn update(&mut self, sd_pt: &SummaryDataPt, cters: Vec<u32>) {
        self.name = sd_pt.name.clone();
        self.league = League::from(sd_pt.get_league());
        self.cters = cters;
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct League {
    pub id: i8,            //段位id
    pub league_score: i32, //段位积分
    pub league_time: i64,  //进入段位的时间
}

unsafe impl Send for League {}

impl League {
    pub fn get_league_id(&self) -> i8 {
        self.id
    }
}

impl From<&LeaguePt> for League {
    fn from(l_pt: &LeaguePt) -> Self {
        let league_id = l_pt.get_league_id() as i8;
        League {
            id: league_id,
            league_time: l_pt.league_time,
            league_score: l_pt.league_score,
        }
    }
}
