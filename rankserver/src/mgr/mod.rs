use tools::protos::base::{LeaguePt, RankInfoPt, SummaryDataPt};

pub mod rank_mgr;

pub struct RankInfoPtr(pub *mut RankInfo);

unsafe impl Send for RankInfoPtr {}

impl RankInfoPtr {
    pub fn update(&mut self, sd_pt: &SummaryDataPt, cters: Vec<u32>) {
        unsafe {
            let res = self.0.as_mut().unwrap();
            res.name = sd_pt.name.clone();
            res.league = League::from(sd_pt.get_league());
            res.cters = cters;
        }
    }
}

///排行榜数据结构体
#[derive(Default, Debug)]
pub struct RankInfo {
    pub user_id: u32,    //玩家id
    pub name: String,    //名字
    pub rank: i32,       //排名
    pub cters: Vec<u32>, //最常用的三个角色
    pub league: League,  //段位
}

unsafe impl Send for RankInfo {}

impl RankInfo {
    pub fn reset(&mut self) {
        self.rank = -1;
        self.league.id = 0;
        self.league.league_time = 0;
        self.league.league_score = 0;
    }

    ///更新段位
    pub fn update_league(&mut self, id: i8) {
        let res = crate::TEMPLATES.league_temp_mgr().get_temp(&id).unwrap();
        self.league.id = res.id;
        let time = chrono::Local::now();
        self.league.league_score = res.score;
        self.league.league_time = time.timestamp_millis();
    }

    pub fn get_insert_sql_str(&self) -> String {
        let mut map = serde_json::Map::new();
        map.insert("id".to_owned(), serde_json::Value::from(self.league.id));
        map.insert(
            "name".to_owned(),
            serde_json::Value::from(self.name.clone()),
        );
        map.insert("rank".to_owned(), serde_json::Value::from(self.rank));
        map.insert(
            "cters".to_owned(),
            serde_json::Value::from(self.cters.as_slice()),
        );
        map.insert(
            "score".to_owned(),
            serde_json::Value::from(self.get_score()),
        );
        map.insert("user_id".to_owned(), serde_json::Value::from(self.user_id));
        let json = serde_json::Value::from(map);
        let res = format!(
            "insert into t_u_last_season_rank(user_id,content) values({},{:?})",
            self.user_id,
            json.to_string()
        );
        res
    }
    pub fn init_from_json(js: serde_json::Value) -> anyhow::Result<Self> {
        let mut ri = RankInfo::default();
        ri.user_id = js["user_id"].as_i64().unwrap() as u32;
        ri.name = js["name"].as_str().unwrap().to_string();
        ri.rank = js["rank"].as_i64().unwrap() as i32;
        let cters = js["cters"].as_array();
        if let Some(cters) = cters {
            for cter in cters {
                let cter_id = cter.as_i64().unwrap() as u32;
                ri.cters.push(cter_id);
            }
        }
        ri.league.id = js["id"].as_i64().unwrap() as i8;
        ri.league.league_score = js["score"].as_i64().unwrap() as i32;

        let time = js["league_time"].as_i64().unwrap();
        ri.league.league_time = time;
        Ok(ri)
    }

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
        rip.set_league_score(self.league.league_score);
        rip
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
}

#[derive(Default, Debug)]
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
