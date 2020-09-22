use std::collections::HashMap;
use std::sync::Mutex;
use fluffy::{model::Db, db};
use crate::config;

/// 状态说明
pub const STATES: [&'static str; 2] = ["禁用", "正常"];

lazy_static! { 
    /// 数据库-表-字段映射关系
    pub static ref TABLE_FIELDS: Mutex<HashMap<String, Vec<String>>> = {
        let mut conn = db::get_conn();
        let setting = &*config::SETTING;
        let info = &setting.database;
        let table_fields = Db::get_table_fields(&mut conn, &info.name);
        Mutex::new(table_fields)
    };
}

pub mod menus;
pub mod admin_roles;
pub mod ads;
pub mod video_categories;
pub mod video_tags;
pub mod video_authors;
