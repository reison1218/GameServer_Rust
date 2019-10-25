pub mod user;
use crate::db::dbtool::DbPool;
use chrono::NaiveTime;
use log::{debug, error, info, warn, LevelFilter, Log, Record};
use mysql::prelude::ToValue;
use mysql::Value;
use std::cell::Cell;
use std::collections::HashMap;
use std::error::Error;
use std::ops::Add;

pub trait Entity: Clone {
    fn to_vec_value(&mut self) -> Vec<Value>;
    fn add_version(&mut self);
    fn clear_version(&mut self);
    fn get_version(&self) -> u32;
}

pub trait dao: Entity {
    fn query(user_id: u32, pool: &mut DbPool) -> Option<Self>;

    fn update(&mut self, pool: &mut DbPool) -> Result<u32, String>;
}
