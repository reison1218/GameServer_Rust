use crate::models::Menus as ThisModel;
use super::Controller;
use crate::caches::menus::{MAIN_MENUS, self};

pub struct Menus { }

impl Controller for Menus { 

    type M = ThisModel;

    fn edit_after(data: &mut tera::Context) {
        data.insert("menus", &*MAIN_MENUS);
    }

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("name", "%"), ("state", "="), ("url", "%"), ("is_blank", "=")]
    }

    fn save_after() { 
        menus::refresh(); //刷新菜单缓存
    }

    fn delete_after() {
        menus::refresh();
    }
}
