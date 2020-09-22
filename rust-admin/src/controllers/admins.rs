use crate::models::Admins as ThisModel;
use super::Controller;
use crate::caches::admin_roles::ADMIN_ROLES;

pub struct Admins { }

impl Controller for Admins { 

    type M = ThisModel;

    fn edit_after(data: &mut tera::Context) {
        let roles = ADMIN_ROLES.lock().unwrap();
        data.insert("roles", &*roles);
    }

    fn index_after(data: &mut tera::Context) { 
        let roles = ADMIN_ROLES.lock().unwrap();
        data.insert("roles", &*roles);
    }

    fn get_query_cond() -> Vec<(&'static str, &'static str)> { 
        vec![("id", "="), 
            ("name", "%"), 
            ("state", "="),
            ("last_ip", "%"), 
            ("created", "[date]"), 
            ("updated", "[date]"), 
            ("role_id", "="),
            ("last_login", "[date]")]
    }
}
