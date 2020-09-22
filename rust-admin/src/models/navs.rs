use fluffy::{DbRow, model::Model,};
use super::ModelBackend;
use serde_derive::{Serialize};
use std::collections::HashMap;
use crate::validations::Validator;

#[derive(Default, Debug, Serialize)]
pub struct Navs { 
    pub id: usize,
    pub name: String,
    pub url: String, 
    pub remark: String,
    pub seq: isize,
    pub is_blank: u32,
}

impl Model for Navs { 
    fn get_table_name() -> &'static str { "navs" }
}

impl ModelBackend for Navs { 

    type M = Self;

    get_fields!(Self, [
        name => String,
        url => String,
        is_blank => u32,
        remark => String,
        seq => isize,
    ]);

    fn validate(data: &HashMap<String, String>) -> Result<(), String> { 
        Validator::load(&data)
            .string_length("name", "分类名称必须在2-20之间", 2, 20, true)
            .string_length("url", "链接地址不能为空", 1, 200, true)
            .is_yes_no("is_blank", "必须选择是否是外链")
            .string_limit("remark", "备注长度不能超过200", 200)
            .is_numeric("seq", "排序必须是数字")
            .validate()
    }
}
