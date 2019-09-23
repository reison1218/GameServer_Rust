use super::*;
pub struct Channel {
    user_id: u32,
    sender : WsSender
}

impl Channel{
    pub fn init(user_id:u32,sender:WsSender)->Self{
        Channel{user_id:user_id,sender:sender}
    }
}


