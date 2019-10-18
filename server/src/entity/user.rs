use super::*;
//use postgres::types::ToSql;

#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub account: String,
    pub channel: String,
    pub platform: String,
    pub gold: f64,
    pub token: String,
    pub login_time: chrono::NaiveDateTime,
}

impl User {
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
        }
    }
}

impl dao for User {
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
}
