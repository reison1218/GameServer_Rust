use crate::models::Ads as ThisModel;
use super::Controller;
use tera::Context;
use crate::caches::ads::{PAGES, POSITIONS};

pub struct Ads {}

impl Controller for Ads { 

    type M = ThisModel;

    fn index_after(data: &mut Context) { 
        data.insert("positions", &*POSITIONS);
        data.insert("pages", &*PAGES);
    }

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("page_id", "="), 
            ("position_id", "="),
            ("name", "%"),
            ("remark", "%"),
            ("is_blank", "="),]
    }
}
