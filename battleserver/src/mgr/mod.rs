pub mod battle_mgr;
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

///段位数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct League {
    pub score: i32,       //段位积分
    pub league_time: i64, //进入段位时间
    pub league_id: i8,    //段位id
}

impl League {
    pub fn get_league_id(&self) -> i8 {
        self.league_id
    }

    pub fn into_pt(&self) -> LeaguePt {
        let mut lp = LeaguePt::new();
        lp.league_id = self.get_league_id() as i32;
        lp.league_score = self.score;
        lp.league_time = self.league_time;
        lp
    }

    pub fn update_score(&mut self, score: i32) -> i32 {
        self.score += score;
        if self.score < 0 {
            self.score = 0;
            return 0;
        }
        let mgr = crate::TEMPLATES.league_temp_mgr();
        if score < 0 {
            let league_temp = mgr.get_temp(&self.league_id);
            if let Err(_) = league_temp {
                self.league_id = 0;
            }
        }

        let league_temp = mgr.get_league_by_score(self.score);
        if league_temp.is_err() {
            return self.score;
        }
        let league_temp = league_temp.unwrap();
        //掉分了不能掉段位
        if self.score < league_temp.score {
            self.score = league_temp.score;
            return self.score;
        }

        if league_temp.id != self.get_league_id() {
            self.league_id = league_temp.id;
            let time = chrono::Local::now();
            self.league_time = time.timestamp_millis();
        }
        self.score
    }
}

impl From<&LeaguePt> for League {
    fn from(pt: &LeaguePt) -> Self {
        let mut l = League::default();
        l.league_time = pt.league_time;
        l.score = pt.league_score;
        l.league_id = pt.league_id as i8;
        l
    }
}
