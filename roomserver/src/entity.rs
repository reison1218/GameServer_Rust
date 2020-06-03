pub mod battle_model;
pub mod map_data;
pub mod member;
pub mod room;
pub mod team;

use error_chain::bail;
use log::error;
use tools::result::errors::Result;
use tools::tcp::*;
