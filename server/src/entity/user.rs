use super::*;
use crate::db::table_contants::{CHARACTER, LEAGUE, USER};
use crate::entity::character::{Character, Characters};
use crate::entity::league::League;
use std::borrow::{Borrow, BorrowMut};
use std::cell::Cell;

///玩家数据封装结构体
#[derive(Debug, Clone, Default)]
pub struct UserData {
    ///玩家基本数据
    pub user_info: User,
    ///玩家角色
    character: Characters,
    ///玩家段位数据
    pub league: League,
    ///版本号（大于0代表有修改，需要update到db）
    version: Cell<u32>,
}

unsafe impl Send for UserData {}
unsafe impl Sync for UserData {}

///为userdata结构体实现一些基础函数
impl UserData {
    pub fn update_off(&mut self) {
        self.user_info.update_off();
        self.add_version();
        self.update();
    }

    ///构造函数，创建一个新的userdata结构体
    pub fn new(user_info: User, character: Characters, league: League) -> UserData {
        UserData {
            user_info,
            character,
            league,
            version: Cell::new(0),
        }
    }

    pub fn init_from_db(user_id: u32) -> Option<Self> {
        //初始化玩家基础数据
        let user = User::query(USER, user_id, None);
        if user.is_none() {
            return None;
        }
        let user = user.unwrap();
        //段位数据
        let mut league = League::query(LEAGUE, user_id);
        if league.is_none() {
            let res = League::new(user.user_id, user.nick_name.clone());
            async_std::task::spawn(insert_league(res.clone()));
            league = Some(res);
        }

        //初始化玩家角色数据
        let mut cters = Characters::query(CHARACTER, user_id);
        if cters.is_none() {
            let c = Characters::new(user_id);
            async_std::task::spawn(insert_characters(c.clone()));
            cters = Some(c);
        }
        let ud = UserData::new(user, cters.unwrap(), league.unwrap());
        Some(ud)
    }
    ///获得玩家id
    pub fn get_user_id(&self) -> u32 {
        self.user_info.user_id
    }
    ///获得数据版本号
    pub fn get_version(&self) -> u32 {
        self.version.get()
    }
    ///清空版本号
    pub fn clear_version(&self) {
        self.version.set(0);
    }

    ///更新函数，update到db
    pub fn update(&mut self) {
        if self.version.get() == 0 {
            return;
        }

        let res = self.user_info.update();
        if let Err(e) = res {
            error!("{:?}", e);
        }

        for cter in self.character.cter_map.values() {
            if cter.get_version() == 0 {
                continue;
            }
            let res = cter.update();
            if let Err(e) = res {
                error!("{:?}", e);
            }
        }
        let res = self.league.update();
        if let Err(e) = res {
            error!("{:?}", e);
        }
        self.clear_version();
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
    ///获得段位的只读指针
    pub fn get_league_ref(&self) -> &League {
        self.league.borrow()
    }

    ///获得段位的可写指针
    pub fn get_league_mut_ref(&self) -> &League {
        self.add_version();
        self.league.borrow()
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
    pub fn add_version(&self) {
        let v = self.version.get() + 1;
        self.version.set(v);
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

pub async fn insert_characters(cter: Characters) {
    info!("玩家角色数据不存在,现在创建新角色:{}", cter.user_id);
    for ct in cter.cter_map.iter() {
        let result = Character::insert(ct.1);
        if let Err(e) = result {
            error!("{:?}", e);
        }
    }
}

pub async fn insert_league(league: League) {
    info!("玩家段位数据不存在,现在创建玩家段位数据:{}", league.user_id);
    let res = League::insert(&league);
    if let Err(e) = res {
        error!("{:?}", e);
    }
}
