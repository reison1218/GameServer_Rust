use crate::models::Configs as ThisModel;
use super::Controller;

pub struct Configs { }

impl Controller for Configs { 

    type M = ThisModel;

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![]
    }
}
