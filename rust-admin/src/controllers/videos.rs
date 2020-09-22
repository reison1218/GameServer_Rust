use crate::models::Videos as ThisModel;
use super::Controller;
use crate::config;
use crate::caches::{video_categories, video_tags, video_authors};

pub struct Videos { }

impl Controller for Videos { 

    type M = ThisModel;

    fn edit_after(data: &mut tera::Context) {
        let setting = &*config::SETTING;
        let info = &setting.oss;
        data.insert("bucket", &info.bucket);
        data.insert("region",  &info.region);
        data.insert("end_point", &info.end_point);

        let categories = &*video_categories::VIDEO_CATEGORIES;
        data.insert("categories", &categories);

        let tags = &*video_tags::VIDEO_TAGS;
        data.insert("tags", &tags);

        let authors = &*video_authors::VIDEO_AUTHORS;
        data.insert("authors", &authors);
    }

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("title", "%"), ("remark", "%"), ("created", "[date]"), ("updated", "[date]"), ("duration", "[]")]
    }
}
