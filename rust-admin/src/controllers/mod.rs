use std::collections::HashMap;
use std::fmt::Debug;
use fluffy::{ tmpl::Tpl, response, model::Model, model::Db, data_set::DataSet, db, cond_builder::CondBuilder, datetime};
use crate::models::ModelBackend;
use actix_web::{HttpResponse, web::{Path, Form}, HttpRequest};
use crate::caches;
use serde::ser::{Serialize};
use actix_session::{Session};
use crate::common::Acl;
use percent_encoding::{percent_decode};

pub trait Controller { 
    
    /// 模型
    type M: ModelBackend + Default + Serialize + Debug;
    
    /// 得到控制器名称
    fn get_controller_name() -> &'static str { 
        Self::M::get_table_name()
    }
    
    /// 得到查询条件
    fn get_query_cond() -> Vec<(&'static str, &'static str)> { vec![] }

    /// 得到最终查询条件
    fn get_cond(queries: &HashMap<&str, &str>) -> CondBuilder { 
        let mut cond = CondBuilder::new();
        let conditions = Self::get_query_cond();
        for c in &conditions { 
            let field = c.0;
            let sign = c.1;
            if let Some(value) = queries.get(field) {
                let value_bytes = value.trim().as_bytes();
                let real_value = if let Ok(v) = percent_decode(value_bytes).decode_utf8() { v } else { continue; };
                if real_value == "" { 
                    continue;
                }
                match sign { 
                    "=" => { cond.eq(field, &real_value); },
                    "!=" => { cond.ne(field, &real_value); },
                    ">" => { cond.gt(field, &real_value); },
                    ">=" => { cond.gte(field, &real_value); },
                    "<" => { cond.lt(field, &real_value); },
                    "<=" => { cond.lte(field, &real_value); },
                    "%" => { cond.like(field, &real_value); },
                    _ => { }
                };
            }
            if sign == "[]" {  //数字区间
                let key1 = format!("{}_start", field);
                let value1 = if let Some(v) = queries.get(key1.as_str()) { v.trim() }  else { continue; };
                let key2 = format!("{}_end", field);
                let value2 = if let Some(v) = queries.get(key2.as_str()) { v.trim() } else { continue; };
                if value1 == "" || value2 == "" { 
                    continue;
                }
                cond.between(field, &value1, &value2);
            }
            if sign == "[date]" {  //日期区间
                let key1 = format!("{}_start", field);
                let value1 = if let Some(v) = queries.get(key1.as_str()) { v.trim() } else { "" };
                if value1 != "" { 
                    let date_str = format!("{} 00:00:00", value1);
                    let timestamp = datetime::from_str(date_str.as_str()).timestamp();
                    cond.gt(field, &timestamp);
                }
                let key2 = format!("{}_end", field);
                let value2 = if let Some(v) = queries.get(key2.as_str()) { v.trim() } else { "" };
                if value2 != "" { 
                    let date_str = format!("{} 00:00:00", value2);
                    let timestamp = datetime::from_str(date_str.as_str()).timestamp();
                    cond.lte(field, &timestamp);
                }
            }
        }

        cond
    }
    
    /// 处理额外的追回数据
    fn index_after(_data: &mut tera::Context) {}
    
    /// 主頁
    fn index(request: HttpRequest, session:Session, tpl: Tpl) -> HttpResponse { 
        if !Acl::check_login(&session) || !Acl::check_auth(&request, &session) { 
            return response::redirect("/index/error");
        }
        let query_string = request.query_string();
        let queries = fluffy::request::get_queries(query_string);
        let query_cond = Self::get_cond(&queries);
        let cond = if query_cond.len() > 0 { Some(&query_cond) } else { None };
        let controller_name = Self::get_controller_name(); //控制器名称
        let info = Self::M::get_records(&request, cond);
        let breads = &*caches::menus::BREADS.lock().unwrap();
        let bread_path = if let Some(v) = breads.get(&format!("/{}", controller_name)) { v } else { "" };
        let mut data = tmpl_data![
            "action_name" => &"index",
            "controller_name" => &controller_name,
            "records" => &info.records,
            "pager" => &info.pager,
            "bread_path" => &bread_path,
        ];
        let conds = Self::get_query_cond();
        for (key, sign) in &conds { 
            if sign == &"[]" || sign == &"[date]" {
                let key1 = format!("{}_start", key);
                let value1 = queries.get(key1.as_str()).unwrap_or(&"");
                data.insert(key1, &value1);
                let key2 = format!("{}_end", key);
                let value2 = queries.get(key2.as_str()).unwrap_or(&"");
                data.insert(key2, &value2);
                continue;
            }
            let value = queries.get(key).unwrap_or(&"");
            //if NUMBERS.is_match(value) {  //如果是数字, 则转换成数字
            //    if let Ok(n) = dbg!(value.parse::<usize>()) { 
            //        data.insert(key.to_owned(), &n);
            //        continue;
            //    } 
            //}
            let value_bytes = value.as_bytes();
            let real_value = if let Ok(v) = percent_decode(value_bytes).decode_utf8() { v } else { continue; };
            data.insert(key.to_owned(), &real_value);
        }
        Self::index_after(&mut data);
        let view_file = &format!("{}/index.html", controller_name);
        render!(tpl, view_file, &data)
    }

    /// 处理编辑时需要展现出来的附加数据
    fn edit_after(_data: &mut tera::Context) {}

    /// 編輯
    fn edit(request: HttpRequest, session: Session, info: Path<usize>, tpl: Tpl) -> HttpResponse { 
        if !Acl::check_login(&session) || !Acl::check_auth(&request, &session) { 
            return response::redirect("/index/error");
        }
        let controller_name = Self::get_controller_name(); //控制器名称
        let id = info.into_inner();
        let is_update = id > 0;
        let row = if !is_update { Self::M::get_default() } else { 
            let fields = Self::M::get_fields();
            let query = query![fields => &fields, ];
            let cond = cond!["id" => id,];
            let mut conn = fluffy::db::get_conn();
            if let Some(r) = Self::M::fetch_row(&mut conn, &query, Some(&cond)) { 
                Self::M::get_record(r)
            } else { Self::M::get_default() }
        };
        let mut data = tmpl_data![
            "controller_name" => controller_name,
            "row" => &row,
            "id" => &id,
        ];
        Self::edit_after(&mut data);
        let view_file = &format!("{}/edit.html", controller_name);
        render!(tpl, view_file, &data)
    }

    /// 編輯
    fn save(request: HttpRequest, session: Session, info: Path<usize>, post: Form<HashMap<String, String>>) -> HttpResponse { 
        if !Acl::check_login(&session) || !Acl::check_auth(&request, &session) { 
            return response::error("拒绝访问, 未授权");
        }
        let id = info.into_inner();
        if id == 0 { Self::save_for_create(post) } else { Self::save_for_update(id, post) }
    }

    /// 添加
    fn save_for_create(post: Form<HashMap<String, String>>) -> HttpResponse { 
        let post_fields = post.into_inner();
        if let Err(message) = Self::M::validate(&post_fields) {  //如果检验出错
            return response::error(message);
        }
        let table_name = Self::M::get_table_name();
        let table_fields = caches::TABLE_FIELDS.lock().unwrap();
        let mut checked_fields = Db::check_fields(table_name, &table_fields, post_fields, false); //經過檢驗之後的數據
        Self::M::save_before(&mut checked_fields); //对于保存数据前的检测
        let mut data = DataSet::create();
        for (k, v) in &checked_fields { 
            data.set(k, &v.trim());
        }
        let mut conn = db::get_conn();
        let id = Self::M::create(&mut conn, &data);
        if id > 0 { 
            Self::save_after();
            return response::ok();
        } 
        response::error("增加記錄失敗")
    }
    
    /// 修改
    fn save_for_update(id: usize, post: Form<HashMap<String, String>>) -> HttpResponse { 
        let post_fields = post.into_inner();
        if let Err(message) = Self::M::validate(&post_fields) {  //如果检验出错
            return response::error(message);
        }
        let table_name = Self::M::get_table_name();
        let table_fields = caches::TABLE_FIELDS.lock().unwrap();
        let mut checked_fields = Db::check_fields(table_name, &table_fields, post_fields, true); //經過檢驗之後的數據
        Self::M::save_before(&mut checked_fields); //对于保存数据前的检测
        let mut data = DataSet::update();
        for (k, v) in &checked_fields { 
            if k == "id" {  //跳过id字段
                continue;
            }
            data.set(k, &v.trim());
        }
        let mut conn = db::get_conn();
        let cond = cond![ "id" => &id, ];
        let id = Self::M::update(&mut conn, &data, &cond);
        if id > 0 { 
            Self::save_after();
            return response::ok();
        } 
        response::error("修改記錄失敗")
    }

    /// 保存之后处理
    fn save_after() { }
    
    /// 刪除
    fn delete(request: HttpRequest, session: Session, id_strings: Path<String>) -> HttpResponse { 
        if !Acl::check_login(&session) || !Acl::check_auth(&request, &session) { 
            return response::error("拒绝访问, 未授权");
        }
        let mut ids_string = String::new();
        for (index, value) in id_strings.split(",").enumerate() { 
            let _ = if let Ok(v) = value.parse::<usize>() { v } else { return response::error("错误的参数"); };
            if index > 0 { 
                ids_string.push_str(",");
            }
            ids_string.push_str(value);
        }
        let cond = cond![
            in_range => ["id" => &ids_string,],
        ];
        let mut conn = db::get_conn();
        let affected_rows = Self::M::delete(&mut conn, &cond);
        if affected_rows == 0 { response::error("未删除任何记录") } else { 
            Self::delete_after();
            response::ok() 
        }
    }

    /// 删除之后处理
    fn delete_after() { }
}

pub mod index;
pub mod admins;
pub mod admin_roles;
pub mod menus;
pub mod users;
pub mod video_categories;
pub mod videos;
pub mod video_replies;
pub mod video_tags;
pub mod user_levels;
pub mod watch_records;
pub mod ads;
pub mod navs;
pub mod configs;
pub mod video_authors;
