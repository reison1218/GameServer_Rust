use super::*;
use crate::db::table_contants;
use crate::db::table_contants::{CHARACTER, USER};
use crate::entity::character::{Character, Characters};
use crate::TEMPLATES;
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use tools::templates::template::TemplateMgrTrait;

///玩家数据封装结构体
#[derive(Debug, Clone, Default)]
pub struct UserData {
    ///玩家基本数据
    user_info: User,
    ///玩家角色
    character: Characters,
    ///版本号（大于0代表有修改，需要update到db）
    version: u32,
}

///为userdata结构体实现一些基础函数
impl UserData {
    ///构造函数，创建一个新的userdata结构体
    pub fn new(user_info: User, character: Characters) -> UserData {
        UserData {
            user_info,
            character: character,
            version: 0,
        }
    }

    pub fn init_from_db(user_id: u32) -> Option<Self> {
        let user = User::query(USER, user_id, None);
        if user.is_none() {
            return None;
        }
        let mut cters = Characters::query(CHARACTER, user_id);
        if cters.is_none() {
            let c = Characters::new(user_id);
            async_std::task::spawn(insert_characters(c.clone()));
            cters = Some(c);
        }
        let ud = UserData::new(user.unwrap(), cters.unwrap());
        Some(ud)
    }

    pub fn init(&mut self, user_info: User, character: Characters) {
        self.user_info = user_info;
        self.character = character;
        self.version = 0 as u32;
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
                Ok(_) => {
                    self.clear_version();
                }
                Err(_) => {}
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

    ///获得character结构体的只读指针
    pub fn get_characters_ref(&self) -> &Characters {
        self.character.borrow()
    }

    ///获得character结构体的可变指针
    pub fn get_characters_mut_ref(&mut self) -> &mut Characters {
        self.add_version();
        self.character.borrow_mut()
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

pub async fn insert_user(mut user: User) {
    info!("玩家数据不存在,现在创建新玩家:{}", user.user_id);
    user.clear_version();
    let result = User::insert(&mut user);
    if result.is_err() {
        error!("{:?}", result.err().unwrap());
    }
}

pub async fn insert_characters(mut cter: Characters) {
    info!("玩家角色数据不存在,现在创建新角色:{}", cter.user_id);
    for ct in cter.cter_map.iter_mut() {
        let result = Character::insert(ct.1);
        if result.is_err() {
            error!("{:?}", result.err().unwrap());
        }
    }
}
