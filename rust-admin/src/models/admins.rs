use fluffy::{DbRow, model::Model, random, utils};
use super::ModelBackend;
use serde_derive::{Serialize};
use crate::validations::Validator;
use std::collections::HashMap;

#[derive(Default, Debug, Serialize)]
pub struct Admins { 
    pub id: usize, //编号
    pub name: String, //用户名称
    pub last_ip: String, //最后登录ip
    pub state: u32, //状态, 是否可用, 0: 不可用, 1:可用
    pub login_count: u32, //登录次数
    pub last_login: u32, //最后登录时间
    pub created: u32, //添加时间
    pub updated: u32, //更新时间
    pub role_id: usize,
    pub seq: isize,
}

impl Model for Admins { 
    fn get_table_name() -> &'static str { "admins" }
}

impl ModelBackend for Admins { 

    type M = Self;

    get_fields!(Self, [
        name => String,
        last_ip => String,
        role_id => usize,
        state => u32,
        login_count => u32,
        last_login => u32,
        created => u32,
        updated => u32,
        seq => isize,
    ]);

    fn save_before(data: &mut HashMap<String, String>) { 
        if let Some(v) = data.get("password") {  //如果提交的有密码
            let secret = random::rand_str(32);
            let password = utils::get_password(&secret, v);
            data.insert("password".to_owned(), password);
            data.insert("secret".to_owned(), secret);
        }
    }

    fn validate(data: &HashMap<String, String>) -> Result<(), String> { 
        let mut vali = Validator::load(&data);
        let mut need_password = false;
        if let Some(v) = data.get("id") { 
            if v == "0" { 
                need_password = true
            } else if let Some(_) = data.get("password") { 
                need_password = true
            }
        } 

        println!("data = {:?}", data);
        if need_password {
            vali.is_password("password", "必须输入密码");
            vali.equal("password", "re_password", "两次输入的密码必须一致");
        }
        vali.is_username("name", "请输入正确格式的用户名称", true)
        .is_yes_no("state", "状态值不正确")
        .is_numeric("seq", "排序必须是有效的数字")
        .validate()
    }

}
