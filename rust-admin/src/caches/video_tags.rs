use std::collections::HashMap;
use std::sync::Mutex;
use fluffy::{db, model::Model};
use crate::models::{VideoTags};

lazy_static! { 
    /// 对全部标签分类进行缓存
    pub static ref VIDEO_TAGS: Mutex<HashMap<usize, String>> = { 
        let rows = get_cache_items();
        Mutex::new(rows)
    };
}

/// 刷新缓存
pub fn refresh() { 
    let mut list = VIDEO_TAGS.lock().unwrap();
    *list = get_cache_items();
}

/// 得到所有的视频分类
fn get_cache_items() -> HashMap<usize, String> { 
    let mut conn = db::get_conn();
    let query = query![
        fields => "id, name",
        limit => 100,
    ];
    let mut list = HashMap::new();
    let rows = VideoTags::fetch_rows(&mut conn, &query, None);
    for r in rows { 
        let (id, name): (usize, String) = from_row!(r);
        list.insert(id, name);
    }
    list
}


