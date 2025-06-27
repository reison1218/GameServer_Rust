pub mod gateway_handler;
pub mod get_json_config;
pub mod http_server;
pub mod test_handler;

use tools::http::HttpServerHandler;
use tools::json::JsonValue;
use tools::json::*;

fn is_valid_datetime(input: &str) -> bool {
    chrono::NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").is_ok()
}
