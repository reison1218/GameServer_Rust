use std::collections::HashMap;
use crate::validations::{Validator};

pub struct Index {}

impl Index { 

    /// 检测用户登录
    pub fn check_login(data: &HashMap<String, String>) -> Result<(), String> { 
        Validator::load(data)
            .is_username("username", "必须输入正确格式的用户名称", true)
            .is_password("password", "必须输入密码")
            .validate()
    }

    /// 检测修改密码
    pub fn check_change_pwd(data: &HashMap<String, String>) -> Result<(), String> { 
        Validator::load(data)
            .is_password("old_password", "必须输入正确格式的旧密码")
            .is_password("password", "必须输入正确格式的密码")
            .is_password("re_password", "必须输入重复密码")
            .equal("password", "re_password", "两次输入的密码必须一致")
            .validate()
    }
}
