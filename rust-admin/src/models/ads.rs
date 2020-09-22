use fluffy::{DbRow, model::Model,};
use super::ModelBackend;
use serde_derive::{Serialize};
use std::collections::HashMap;
use crate::validations::Validator;
use crate::caches::ads::{PAGE_IDS, POSITION_IDS};

#[derive(Default, Debug, Serialize)]
pub struct Ads { 
    pub id: usize,
    pub name: String,
    pub remark: String, 
    pub image: String,
    pub page_id: u32,
    pub position_id: u32,
    pub url: String,
    pub is_blank: u32,
    pub seq: isize,
}

impl Model for Ads { 
    fn get_table_name() -> &'static str { "ads" }
}

impl ModelBackend for Ads { 

    type M = Self;

    get_fields!(Self, [
        name => String,
        remark => String,
        image => String,
        page_id => u32,
        position_id => u32,
        url => String,
        is_blank => u32,
        seq => isize,
    ]);

    fn validate(data: &HashMap<String, String>) -> Result<(), String> { 
        Validator::load(&data)
            .string_length("name", "名称必须在2-20之间", 2, 20, true)
            .string_limit("remark", "备注长度不能超过200", 200)
            .string_limit("image", "图片地址不能超过200字符", 200)
            .in_range("page_id", "必须在可选范围之内", &PAGE_IDS)
            .in_range("position_id", "位置必须在范围之内", &POSITION_IDS)
            .string_limit("url", "链接地址长度不能超过200", 200)
            .is_yes_no("is_blank", "必须输入选项是否外链")
            .is_numeric("seq", "排序必须是有效的数字")
            .validate()
    }
}
