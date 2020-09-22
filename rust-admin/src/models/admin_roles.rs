use fluffy::{DbRow, model::Model,};
use super::ModelBackend;
use serde_derive::{Serialize};
use crate::validations::Validator;
use std::collections::HashMap;

#[derive(Default, Debug, Serialize)]
pub struct AdminRoles { 
    pub id: usize, //编号
    pub name: String, //名称
    pub remark: String, //备注
    pub seq: isize, //
    pub menu_ids: String,
}

impl Model for AdminRoles { 
    fn get_table_name() -> &'static str { "admin_roles" }
}

impl ModelBackend for AdminRoles { 

    type M = Self;

    get_fields!(Self, [
        name => String,
        remark => String,
        seq => isize,
        menu_ids => String,
    ]);

    fn validate(data: &HashMap<String, String>) -> Result<(), String> { 
        Validator::load(&data)
            .string_length("name", "分类名称必须在2-20之间", 2, 20, true)
            .string_limit("remark", "备注长度必须在0-50之间", 50)
            .is_numeric("seq", "排序必须是有效的数字")
            .validate()
    }
}
