use super::*;

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: u32,    //玩家id
    pub data: JsonValue, //数据
    version: Cell<u32>,  //数据版本号
}

impl Data for User {
    fn get_json_value(&mut self, key: &str) -> Option<&JsonValue> {
        let v = self.data.as_object_mut().unwrap().get(key);
        if v.is_none() {
            return None;
        }
        v
    }

    fn get_mut_json_value(&mut self) -> Option<&mut Map<String, JsonValue>> {
        self.data.as_object_mut()
    }

    ///获取玩家id
    fn get_id(&self) -> Option<u32> {
        Some(self.user_id)
    }

    ///设置玩家id
    fn set_id(&mut self, id: u32) {
        self.user_id = id;
        self.add_version();
    }
}

impl Dao for User {
    ///查询函数
    fn query(user_id: u32, pool: &mut DbPool) -> Option<Self> {
        let mut v: Vec<Value> = Vec::new();
        v.push(Value::Int(user_id as i64));
        let mut q = pool
            .exe_sql("select * from t_u_player where UserId=:id", Some(v))
            .unwrap();

        let mut user: Option<User> = None;
        for _qr in q {
            let (id, js) = mysql::from_row(_qr.unwrap());
            let mut u = User::init(id, js);
            println!("查询时间:{}", u.get_time("login_time").unwrap());
            user = Some(u);
        }
        user
    }

    fn update(&mut self, pool: &mut DbPool) -> Result<u32, String> {
        let mut v: Vec<Value> = self.to_vec_value();
        let mut qr = pool.exe_sql(
            "update t_u_player set Content=:data where UserId=:user_id",
            Some(v),
        );
        let mut str = String::from_utf8(qr.unwrap().info());
        println!("{:?}", str.unwrap());

        Ok(1)
    }
}

impl Entity for User {
    fn to_vec_value(&mut self) -> Vec<Value> {
        let mut v: Vec<Value> = Vec::new();
        v.push(self.user_id.to_value());
        v.push(self.data.as_str().to_value());
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

    ///初始化函数
    /// 返回一个User结构体
    fn init(id: u32, js: JsonValue) -> Self {
        User {
            user_id: id,
            data: js,
            version: Cell::new(0),
        }
    }
}
