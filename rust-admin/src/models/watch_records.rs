use fluffy::{DbRow, model::Model,};
use serde_derive::{Serialize};
use super::ModelBackend;

#[derive(Default, Debug, Serialize)]
pub struct WatchRecords { 
    pub id: usize,
    pub video_id: usize,
    pub user_id: usize,
    pub user_name: String,
    pub created: u32,
}

impl Model for WatchRecords { 
    fn get_table_name() -> &'static str { "watch_records" }
}

impl ModelBackend for WatchRecords { 

    type M = Self;

    get_fields!(Self, [
        video_id => usize,
        user_id => usize,
        user_name => String,
        created => u32,
    ]);
}
