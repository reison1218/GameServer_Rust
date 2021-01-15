use tools::protos::base::{LeaguePt, SummaryDataPt};

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
    pub rank: u32,       //排名
    pub cters: Vec<u32>, //最常用的三个角色
    pub league: League,  //段位
}

impl RankInfo {
    ///获得积分
    pub fn get_score(&self) -> i32 {
        self.league.league_score
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
