use crate::models::WatchRecords as ThisModel;
use super::Controller;

pub struct WatchRecords { }

impl Controller for WatchRecords { 

    type M = ThisModel;

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("user_id", "="), ("user_name", "%"), ("video_id", "="), ("created", "[date]")]
    }
}
