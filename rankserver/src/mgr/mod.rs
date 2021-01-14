use log::warn;
use tools::templates::league_temp::LeagueTemp;
pub mod rank_mgr;

pub struct RankInfoPtr(pub *mut RankInfo);

unsafe impl Send for RankInfoPtr {}

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

    pub fn new(user_id: u32, name: String) -> Self {
        let league = League::new(10).unwrap();

        RankInfo {
            user_id,
            name,
            rank: 0,
            cters: Vec::new(),
            league,
        }
    }

    ///获得段位数据
    pub fn get_league_id(&self) -> u8 {
        self.league.temp.id
    }

    ///更新积分
    pub fn update(&mut self, add_score: i32) {
        let before_score = self.get_score();
        let mut res_score = self.league.league_score;
        res_score = res_score.saturating_add(add_score);
        if res_score < 0 {
            res_score = 0;
        }
        if res_score < before_score && res_score < self.league.temp.score {
            self.league.league_score = self.league.temp.score;
            return;
        } else if res_score > before_score {
            let mgr = crate::TEMPLATES.get_league_temp_mgr_ref();
            let res = mgr.get_league_by_score(res_score);
            if let Err(e) = res {
                warn!("{:?}", e);
                return;
            }
            let temp = res.unwrap();
            if self.get_league_id() != temp.id {
                self.league.temp = temp;
            }
            self.league.league_score = res_score;
        }
    }
}

#[derive(Debug)]
pub struct League {
    pub temp: &'static LeagueTemp, //段位id
    pub league_score: i32,         //段位积分
    pub league_time: u64,          //进入段位的时间
}

impl League {
    pub fn new(league_score: i32) -> anyhow::Result<Self> {
        let mgr = crate::TEMPLATES.get_league_temp_mgr_ref();
        let res = mgr.get_league_by_score(league_score);
        if let Err(e) = res {
            anyhow::bail!("{:?}", e)
        }
        let temp = res.unwrap();
        let res = League {
            temp,
            league_score,
            league_time: 0,
        };
        Ok(res)
    }

    pub fn get_league_id(&self) -> u8 {
        self.temp.id
    }
}
