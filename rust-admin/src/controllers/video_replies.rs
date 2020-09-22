use crate::models::VideoReplies as ThisModel;
use super::Controller;

pub struct VideoReplies { }

impl Controller for VideoReplies { 

    type M = ThisModel;

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("video_id", "="), 
            ("reply_id", "="), 
            ("user_id", "="), 
            ("user_name", "%"), 
            ("created", "[date]"),
            ("content", "%")] 
    }
}
