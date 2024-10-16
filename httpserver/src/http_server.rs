use crate::handler::add_server_handler::AddServerHandler;
use crate::handler::game_yearly_handler::GameYearlyHandler;
use crate::handler::modify_server_handler::ModifyServerHandler;
use crate::handler::modify_white_user_handler::ModifyWhiteUserHandler;
use crate::handler::questionnaire_handler::QuestionnaireHandler;
use crate::handler::reload_handler::ReloadHandler;
use crate::handler::server_list_handler::ServerListHandler;
use crate::handler::test_handler::TestHandler;
use crate::handler::wx_game_message_handler::NoticeMessHandler;
use crate::handler::wx_game_subscribe_handler::WxGameSubscribeHandler;

pub fn init_server() {
    let port = crate::CONF_MAP.get_usize("http_listen_port", 8080);
    tools::http::Builder::new()
        .route(Box::new(TestHandler))
        .route(Box::new(WxGameSubscribeHandler))
        .route(Box::new(NoticeMessHandler))
        .route(Box::new(ModifyServerHandler))
        .route(Box::new(AddServerHandler))
        .route(Box::new(QuestionnaireHandler))
        .route(Box::new(ModifyWhiteUserHandler))
        .route(Box::new(GameYearlyHandler))
        .route(Box::new(ServerListHandler))
        .route(Box::new(ReloadHandler))
        .bind(port as u16);
}
