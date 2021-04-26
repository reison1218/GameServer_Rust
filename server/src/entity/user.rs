use super::*;
use crate::db::table_contants::{CHARACTER, GRADE_FRAME, SOUL, USER};
use crate::entity::character::{Character, Characters};
use crate::entity::grade_frame::GradeFrame;
use crate::entity::soul::Soul;
use std::borrow::{Borrow, BorrowMut};
use std::cell::Cell;

///玩家数据封装结构体
#[derive(Debug, Clone, Default)]
pub struct UserData {
    ///玩家基本数据
    pub user_info: User,
    ///玩家角色
    character: Characters,
    ///玩家grade相框
    pub grade_frame: GradeFrame,
    ///灵魂头像
    pub soul: Soul,
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
    pub fn new(
        user_info: User,
        character: Characters,
        grade_frame: GradeFrame,
        soul: Soul,
    ) -> UserData {
        UserData {
            user_info,
            character,
            grade_frame,
            soul,
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

        let mut gf = GradeFrame::query(GRADE_FRAME, user_id);
        //grade相框数据
        if gf.is_none() {
            let res = GradeFrame::new(user.user_id);
            async_std::task::spawn(insert_grade_frame(res.clone()));
            gf = Some(res);
        }
        let mut soul = Soul::query(SOUL, user_id);
        //soul头像数据
        if soul.is_none() {
            let res = Soul::new(user.user_id);
            async_std::task::spawn(insert_soul(res.clone()));
            soul = Some(res);
        }
        //初始化玩家角色数据
        let mut cters = Characters::query(CHARACTER, user_id);
        if cters.is_none() {
            let c = Characters::new(user_id);
            async_std::task::spawn(insert_characters(c.clone()));
            cters = Some(c);
        }
        let ud = UserData::new(user, cters.unwrap(), gf.unwrap(), soul.unwrap());
        Some(ud)
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
        let res = self.soul.update();
        if let Err(e) = res {
            error!("{:?}", e);
        }
        let res = self.grade_frame.update();
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

pub async fn insert_grade_frame(grade_frame: GradeFrame) {
    info!(
        "玩家grade头像框数据不存在,现在创建玩家段位数据:{}",
        grade_frame.user_id
    );
    let res = GradeFrame::insert(&grade_frame);
    if let Err(e) = res {
        error!("{:?}", e);
    }
}

pub async fn insert_soul(soul: Soul) {
    info!("玩家Soul数据不存在,现在创建玩家段位数据:{}", soul.user_id);
    let res = Soul::insert(&soul);
    if let Err(e) = res {
        error!("{:?}", e);
    }
}
