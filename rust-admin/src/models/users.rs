use std::collections::HashMap;
use fluffy::{DbRow, model::Model,};
use super::ModelBackend;
use serde_derive::{Serialize};
use crate::validations::Validator;

#[derive(Default, Debug, Serialize)]
pub struct Users { 
    pub id: usize, //编号
    pub name: String, //用户名称
    pub last_ip: String, //最后登录ip
    pub state: u32, //状态, 是否可用, 0: 不可用, 1:可用
    pub login_count: u32, //登录次数
    pub last_login: u32, //最后登录时间
    pub remark: String,
    pub created: u32, //添加时间
    pub updated: u32, //更新时间
}

impl Model for Users { 
    fn get_table_name() -> &'static str { "users" }
}

impl ModelBackend for Users { 

    type M = Self;

    get_fields!(Self, [
        name => String,
        last_ip => String,
        state => u32,
        login_count => u32,
        last_login => u32,
        remark => String,
        created => u32,
        updated => u32,
    ]);

    fn validate(data: &HashMap<String, String>) -> Result<(), String> { 
        Validator::load(&data)
            .is_username("name", "必须是用户名称", true)
            .validate()
    }
}
