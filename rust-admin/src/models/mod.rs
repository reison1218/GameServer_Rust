use std::collections::HashMap;
use std::default::Default;
use std::fmt::Debug;
use fluffy::{ db, DbRow, Pager, model::Model, cond_builder::CondBuilder };
use serde::ser::Serialize;
use actix_web::{HttpRequest};

#[derive(Debug, Default)]
pub struct DataGrid<M: Model + Serialize> { 
    pub records: Vec<M>,
    pub pager: Pager,
}

//{
//	"code": 0,
//	"success": true,
//	"msg": "签名成功",
//	"data": {
//		"accessid": "XXXXX",
//		"host": "http://XXXXX.oss-cn-shanghai.aliyuncs.com",
//		"policy": "XXXX==",
//		"signature": "XXXX=",
//		"expire": 1554851252
//	}
//}

#[derive(Serialize)]
pub struct UploadResult<'a> { 
    pub code: usize, //错误代码, 0:表示成功
    pub message: &'a str, //错误信息
    pub path: &'a str, //上传的文件的路径
}

#[derive(Serialize)]
pub struct OSSData<'a> { 
    pub access_id: &'a str,
    pub host: &'a str,
    pub policy: &'a str,
    pub signature: &'a str,
    pub expire: u64,
}

/// oss返回的地址
#[derive(Serialize)]
pub struct OSSResult<'a> { 
    pub code: usize,
    pub success: bool,
    pub msg: &'static str,
    pub data: OSSData<'a>,
}

#[macro_export]
macro_rules! get_fields {
    ($struct: ident, [$($field: ident => $type: ident,)+]) => {
        
        /// 得到所有列表字段
        fn get_fields() -> &'static str { 
            concat!("id", $(",", stringify!($field)),+)
        }
    
        /// 得到单条记录
        fn get_record(r: DbRow) -> Self { 
            let mut row = Self::default();
            let (id, $($field),+): (usize, $($type),+) = from_row!(r);
            row.id = id;
            $(row.$field = $field;)+
            row
        }
    }
}


pub trait ModelBackend: Model { 
    
    /// 模型
    type M: Model + Serialize + Default + Debug;

    /// 得到要從資料庫中提取的列頭
    fn get_fields() -> &'static str;

    /// 得到單條記錄
    fn get_record(_: DbRow) -> Self::M;
    
    /// 保存到数据库之前的处理
    fn save_before(_data: &mut HashMap<String, String>) {}

    /// 得到当前的Model
    fn get_default() -> Self::M { Self::M::default() }

    /// 校验提交上来的数据
    fn validate(_data: &HashMap<String, String>) -> Result<(), String>{ Ok(()) }

    /// 得到所有記錄-帶分頁信息
    fn get_records(request: &HttpRequest, cond: Option<&CondBuilder>) -> DataGrid<Self::M> { 
        let fields = Self::get_fields();
        let mut query = query![
            fields => fields,
        ];
        query.set_limit_offset(&request);
        let mut conn = db::get_conn();
        let rows = Self::M::fetch_rows(&mut conn, &query, cond);
        let mut rs: Vec<Self::M> = vec![];
        for r in rows { 
            rs.push(Self::get_record(r));
        }
        let pager = Self::M::get_pager(&mut conn, &query, cond);
        DataGrid { 
            records: rs,
            pager: pager,
        }
    }
}

mod admins;
mod menus;
mod admin_roles;
mod users;
mod videos;
mod video_categories;
mod video_replies;
mod video_tags;
mod user_levels;
mod watch_records;
mod ads;
mod index;
mod navs;
mod configs;
mod video_authors;

pub use admins::Admins;
pub use menus::{Menus, MainMenu, SubMenu};
pub use admin_roles::AdminRoles;
pub use videos::Videos;
pub use video_categories::{VideoCategories};
pub use video_replies::VideoReplies;
pub use users::Users;
pub use user_levels::UserLevels;
pub use video_tags::{VideoTags};
pub use watch_records::WatchRecords;
pub use ads::Ads;
pub use index::Index;
pub use navs::Navs;
pub use configs::Configs;
pub use video_authors::VideoAuthors;
