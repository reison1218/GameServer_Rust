use super::*;
use crate::db::table_contants;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;

///玩家数据封装结构体
#[derive(Debug, Clone, Default)]
pub struct UserData {
    ///玩家基本数据
    user_info: User,
    ///版本号（大于0代表有修改，需要update到db）
    version: u32,
}

///为userdata结构体实现一些基础函数
impl UserData {
    ///构造函数，创建一个新的userdata结构体
    pub fn new(user_info: User) -> UserData {
        UserData {
            user_info,
            version: 0,
        }
    }
    ///获得玩家id
    pub fn get_user_id(&self) -> u32 {
        self.user_info.user_id
    }
    ///获得数据版本号
    pub fn get_version(&self) -> u32 {
        self.version
    }
    ///清空版本号
    pub fn clear_version(&mut self) {
        self.version = 0;
    }

    ///更新函数，update到db
    pub fn update(&mut self) {
        if self.user_info.version > 0 {
            let res = self.user_info.update();
            match res {
                Ok(i) => {
                    self.clear_version();
                }
                Err(e) => {}
            }
        }
    }

    ///获得userinfo结构体的只读指针
    pub fn get_user_info_ref(&self) -> &User {
        self.user_info.borrow()
    }

    ///获得userinfo结构体的可变指针
    pub fn get_user_info_mut_ref(&mut self) -> &mut User {
        self.add_version();
        self.user_info.borrow_mut()
    }

    ///每日重制函数
    pub fn day_reset(&mut self) {
        self.user_info.day_reset();
    }

    ///添加数据版本号
    pub fn add_version(&mut self) {
        self.version += 1;
    }
}
