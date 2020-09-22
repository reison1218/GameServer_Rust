use crate::models::AdminRoles as ThisModel;
use super::Controller;
use crate::models::Menus;
use crate::caches::admin_roles;

pub struct AdminRoles { }

impl Controller for AdminRoles { 

    type M = ThisModel;

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("name", "%"), ("remark", "%")]
    }

    fn edit_after(data: &mut tera::Context) { 
        data.insert("menus", &Menus::get_related());
    }

    fn save_after() { 
        admin_roles::refresh(); //刷新缓存
    }

    fn delete_after() { 
        admin_roles::refresh();
    }
}
