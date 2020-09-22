use crate::models::VideoTags as ThisModel;
use super::Controller;
use crate::caches::video_tags;

pub struct VideoTags {}

impl Controller for VideoTags { 

    type M = ThisModel;

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("name", "%"), ("remark", "%")]
    }

    fn save_after() { 
        video_tags::refresh();
    }

    fn delete_after() { 
        video_tags::refresh();
    }
}   
