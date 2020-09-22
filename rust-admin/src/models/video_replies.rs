use fluffy::{DbRow, model::Model,};
use super::ModelBackend;
use serde_derive::{Serialize};

#[derive(Default, Debug, Serialize)]
pub struct VideoReplies { 
    pub id: usize,
    pub video_id: usize,
    pub reply_id: usize,
    pub user_id: usize, 
    pub user_name: String,
    pub content: String,
    pub created: u32,
}

impl Model for VideoReplies { 
    fn get_table_name() -> &'static str { "video_replies" }
}

impl ModelBackend for VideoReplies { 

    type M = Self;

    get_fields!(Self, [
        video_id => usize,
        reply_id => usize,
        user_id => usize,
        user_name => String,
        content => String,
        created => u32,
    ]);
}
