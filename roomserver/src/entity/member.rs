use super::*;
pub struct Member{
    user_id:u32,
}

impl Member{
    fn new(user_id:u32)->Member{
        Member{user_id}
    }
}