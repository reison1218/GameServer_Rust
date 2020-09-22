use fluffy::{DbRow, model::Model,};
use super::ModelBackend;
use serde_derive::{Serialize};
use std::collections::HashMap;
use crate::validations::Validator;

#[derive(Default, Debug, Serialize)]
pub struct UserLevels { 
    pub id: usize,
    pub name: String,
    pub remark: String, 
    pub watch_per_day: usize,
    pub score_min: u32,
    pub score_max: u32,
    pub seq: isize,
}

impl Model for UserLevels { 
    fn get_table_name() -> &'static str { "user_levels" }
}

impl ModelBackend for UserLevels { 

    type M = Self;

    get_fields!(Self, [
        name => String,
        remark => String,
        watch_per_day => usize,
        score_min => u32,
        score_max => u32,
        seq => isize,
    ]);

    fn validate(data: &HashMap<String, String>) -> Result<(), String> { 
        Validator::load(&data)
            .string_length("name", "分类名称必须在2-20之间", 2, 20, true)
            .validate()
    }
}
