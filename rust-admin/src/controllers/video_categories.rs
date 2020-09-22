use crate::models::VideoCategories as ThisModel;
use super::Controller;
use crate::caches::video_categories;

pub struct VideoCategories { }

impl Controller for VideoCategories { 

    type M = ThisModel;

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("name", "%"), ("remark", "%")]
    }

    fn save_after() { 
        video_categories::refresh();
    }

    fn delete_after() { 
        video_categories::refresh();
    }
}
