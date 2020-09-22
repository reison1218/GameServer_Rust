use crate::models::UserLevels as ThisModel;
use super::Controller;

pub struct UserLevels {}

impl Controller for UserLevels { 

    type M = ThisModel;

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("name", "%"), ("remark", "%")]
    }
}
