use super::*;

pub struct ChannelMgr {
    pub user_channel: HashMap<u32, Channel>,
    pub channels: HashMap<usize, u32>,
}

impl ChannelMgr {
    pub fn new() -> ChannelMgr {
        let mut map = HashMap::new();
        let mut channels = HashMap::new();
        ChannelMgr {
            user_channel: map,
            channels: channels,
        }
    }

    pub fn insert_user_channel(&mut self, k: u32, v: Channel) {
        self.user_channel.insert(k, v);
    }
    pub fn insert_channels(&mut self, k: usize, v: u32) {
        self.channels.insert(k, v);
    }

    pub fn get_user_channel(&mut self, k: &u32) -> Option<&Channel> {
        self.user_channel.get(k)
    }

    pub fn get_channels(&mut self, k: &usize) -> Option<&u32> {
        self.channels.get(k)
    }

    pub fn get_mut_user_channel(&mut self, k: &u32) -> Option<&mut Channel> {
        self.user_channel.get_mut(k)
    }

    pub fn get_mut_channels(&mut self, k: &usize) -> Option<&mut u32> {
        self.channels.get_mut(k)
    }

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
        channel.unwrap().sender.close(CloseCode::Invalid);
        self.user_channel.remove(&user_id.unwrap());
        println!("玩家断开连接，关闭句柄释放资源：{}", user_id.unwrap());
    }
}
