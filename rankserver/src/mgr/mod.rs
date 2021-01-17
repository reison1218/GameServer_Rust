use tools::protos::{
    base::{LeaguePt, SummaryDataPt},
    server_protocol::RankInfoPt,
};

pub mod rank_mgr;

pub struct RankInfoPtr(pub *mut RankInfo);

unsafe impl Send for RankInfoPtr {}

impl RankInfoPtr {
    pub fn update(&mut self, sd_pt: &SummaryDataPt) {
        unsafe {
            let res = self.0.as_mut().unwrap();
            res.name = sd_pt.name.clone();
            res.league = League::from(sd_pt.get_league());
        }
    }
}

///排行榜数据结构体
#[derive(Debug)]
pub struct RankInfo {
    pub user_id: u32,    //玩家id
    pub name: String,    //名字
    pub rank: i32,       //排名
    pub cters: Vec<u32>, //最常用的三个角色
    pub league: League,  //段位
}

unsafe impl Send for RankInfo {}

impl RankInfo {
    ///获得积分
    pub fn get_score(&self) -> i32 {
        self.league.league_score
    }

    pub fn into_rank_pt(&self) -> RankInfoPt {
        let mut rip = RankInfoPt::new();
        rip.user_id = self.user_id;
        rip.name = self.name.clone();
        rip.rank = self.rank;
        rip.set_cters(self.cters.clone());
        rip.set_league_id(self.league.get_league_id() as u32);
        rip
    }
}

impl From<&SummaryDataPt> for RankInfo {
    fn from(sd_pt: &SummaryDataPt) -> Self {
        let league = League::from(sd_pt.get_league());
        RankInfo {
            user_id: sd_pt.user_id,
            name: sd_pt.name.clone(),
            rank: 0,
            cters: Vec::new(),
            league,
        }
    }
}

#[derive(Debug)]
pub struct League {
    pub id: u8,            //段位id
    pub league_score: i32, //段位积分
    pub league_time: i64,  //进入段位的时间
}

unsafe impl Send for League {}

impl League {
    pub fn get_league_id(&self) -> u8 {
        self.id
    }
}

impl From<&LeaguePt> for League {
    fn from(l_pt: &LeaguePt) -> Self {
        let league_id = l_pt.get_league_id() as u8;
        League {
            id: league_id,
            league_time: l_pt.league_time,
            league_score: l_pt.league_score as i32,
        }
    }
}
