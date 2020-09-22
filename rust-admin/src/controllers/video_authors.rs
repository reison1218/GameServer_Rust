use crate::models::VideoAuthors as ThisModel;
use super::Controller;
use crate::caches::video_authors;

pub struct VideoAuthors { }

impl Controller for VideoAuthors { 

    type M = ThisModel;

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("name", "%"), ("remark", "%")]
    }

    fn save_after() { 
        video_authors::refresh();
    }

    fn delete_after() { 
        video_authors::refresh();
    }
}
