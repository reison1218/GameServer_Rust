pub mod add_server_handler;
pub mod game_yearly_handler;
pub mod modify_server_handler;
pub mod modify_white_user_handler;
pub mod questionnaire_handler;
pub mod test_handler;
pub mod wx_game_message_handler;
pub mod wx_game_subscribe_handler;

use serde_json::json;
use std::collections::HashMap;
use tools::http::HttpServerHandler;
use tools::json::JsonValue;
use tools::json::*;
