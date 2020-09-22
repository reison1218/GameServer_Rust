use crate::models::Navs as ThisModel;
use super::Controller;

pub struct Navs { }

impl Controller for Navs { 

    type M = ThisModel;

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("name", "%"), ("remark", "%"), ("url", "%"), ("is_blank", "=")]
    }
}
