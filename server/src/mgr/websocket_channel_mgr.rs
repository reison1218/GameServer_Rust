use super::*;

pub struct WebSocketChannelMgr {
    //user_id,channel
    user_channel: HashMap<u32, Channel>,
    //token,user_id
    channels: HashMap<usize, u32>,
}

impl WebSocketChannelMgr {
    pub fn new() -> WebSocketChannelMgr {
        let mut map = HashMap::new();
        let mut channels = HashMap::new();
        WebSocketChannelMgr {
            user_channel: map,
            channels: channels,
        }
    }

    ///插入channel,key：userid,v:channel
    pub fn insert_user_channel(&mut self, k: u32, v: Channel) {
        self.user_channel.insert(k, v);
    }
    ///插入token-userid的映射
    pub fn insert_channels(&mut self, k: usize, v: u32) {
        self.channels.insert(k, v);
    }
    ///获得玩家channel k:userid
    pub fn get_user_channel(&mut self, k: &u32) -> Option<&Channel> {
        self.user_channel.get(k)
    }

    ///根据token获得userid
    pub fn get_channels(&mut self, k: &usize) -> Option<&u32> {
        self.channels.get(k)
    }

    ///通过userid获得channel
    pub fn get_mut_user_channel(&mut self, k: &u32) -> Option<&mut Channel> {
        self.user_channel.get_mut(k)
    }

    ///通过token获得userid
    pub fn get_mut_channels(&mut self, k: &usize) -> u32 {
        if self.channels.get(k).is_none() {
            return 0;
        }
        *self.channels.get(k).unwrap()
    }

    ///关闭channel句柄，并从内存中删除
    pub fn close_remove(&mut self, k: &usize) {
        let user_id = self.channels.remove(k);
        if user_id.is_none() {
            return;
        }
        println!("玩家断线，{}", user_id.unwrap());
        let channel = self.user_channel.get(&user_id.unwrap());
        if channel.is_none() {
            return;
        }
        channel.unwrap().sender.close(CloseCode::Invalid).unwrap();
        self.user_channel.remove(&user_id.unwrap());
        println!("玩家断开连接，关闭句柄释放资源：{}", user_id.unwrap());
    }
}
