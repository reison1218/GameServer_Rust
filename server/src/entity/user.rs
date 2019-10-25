use super::*;

#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,                           //玩家id
    pub account: String,                   //账号
    pub channel: String,                   //渠道
    pub platform: String,                  //平台
    pub gold: f64,                         //金币
    pub token: String,                     //token
    pub login_time: chrono::NaiveDateTime, //创建时间
    version: Cell<u32>,                    //数据版本号
}

impl User {
    fn get_id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id;
        self.add_version();
    }

    fn get_account(&self) -> &str {
        self.account.as_str()
    }

    fn set_account(&mut self, account: String) {
        self.account = account;
        self.add_version();
    }

    fn get_channel(&self) -> &str {
        self.channel.as_str()
    }

    fn set_channel(&mut self, channel: String) {
        self.channel = channel;
        self.add_version();
    }

    fn get_platform(&self) -> &str {
        self.platform.as_str()
    }

    fn set_platform(&mut self, platform: String) {
        self.platform = platform;
        self.add_version();
    }

    fn get_gold(&self) -> f64 {
        self.gold
    }

    fn set_gold(&mut self, gold: f64) {
        self.gold = gold;
        self.add_version();
    }

    fn get_token(&self) -> &str {
        self.token.as_str()
    }

    fn set_token(&mut self, token: String) {
        self.token = token;
        self.add_version();
    }

    fn get_login_time(&mut self) -> NaiveTime {
        self.login_time.time()
    }

    ///初始化函数
    /// 返回一个User结构体
    fn init(
        id: u32,
        account: String,
        channel: String,
        platform: String,
        gold: f64,
        token: String,
        login_time: chrono::NaiveDateTime,
    ) -> User {
        User {
            id: id,
            account: account,
            channel: channel,
            platform: platform,
            gold: gold,
            token: token,
            login_time: login_time,
            version: Cell::new(0),
        }
    }
}

impl dao for User {
    ///查询函数
    fn query(user_id: u32, pool: &mut DbPool) -> Option<Self> {
        let mut v: Vec<Value> = Vec::new();
        v.push(Value::Int(user_id as i64));
        let mut q = pool
            .exe_sql("select * from t_u_user_test where id=:id", Some(v))
            .unwrap();

        let mut user: Option<User> = None;
        for _qr in q {
            let (id, account, channel, platform, gold, token, login_time) =
                mysql::from_row(_qr.unwrap());
            let u = User::init(id, account, channel, platform, gold, token, login_time);
            println!("查询时间:{}", u.login_time);
            user = Some(u);
        }
        user
    }

    fn update(&mut self, pool: &mut DbPool) -> Result<u32, String> {
        let mut v: Vec<Value> = self.to_vec_value();
        let mut qr = pool.exe_sql("update t_u_user_test set id=:id,account=:account,channel=:channel,platform:platform,gold:gold,token=:token,login_time=:login_time",Some(v));
        let mut str = String::from_utf8(qr.unwrap().info());
        println!("{:?}", str.unwrap());

        Ok(1)
    }
}

impl Entity for User {
    fn to_vec_value(&mut self) -> Vec<Value> {
        let mut v: Vec<Value> = Vec::new();
        v.push(self.id.to_value());
        v.push(self.account.to_value());
        v.push(self.channel.to_value());
        v.push(self.platform.to_value());
        v.push(self.gold.to_value());
        v.push(self.token.to_value());
        v.push(self.login_time.to_value());
        v
    }

    fn add_version(&mut self) {
        self.version.get_mut().add(1);
    }
    fn clear_version(&mut self) {
        self.version.replace(0);
    }
    fn get_version(&self) -> u32 {
        self.version.get()
    }
}
