use crate::models::Users as ThisModel;
use super::Controller;

pub struct Users { }

impl Controller for Users { 

    type M = ThisModel;

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("id", "="), 
            ("name", "%"),
            ("state", "="),
            ("last_login", "[date]"),
            ("created", "[date]"),
            ("updated", "[date]"),]
    }
}
