pub mod dbtool;
use chrono::prelude::*;
use mysql::prelude::ToValue;
use mysql::{Error, Params, Pool, QueryResult, Value};
//use postgres::{Connection, TlsMode};
use log::info;
use std::option::Option::{None, Some};
use std::result::Result;
