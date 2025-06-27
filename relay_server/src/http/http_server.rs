use crate::http::gateway_handler::GatewayHandler;
use crate::http::get_json_config::GetJsonConfigHandler;
use crate::http::test_handler::TestHandler;

pub fn init_server() {
    let port = crate::CONF_MAP.get_usize("http_listen_port", 8500);
    tools::http::Builder::new()
        .route(Box::new(TestHandler))
        .route(Box::new(GatewayHandler))
        .route(Box::new(GetJsonConfigHandler))
        .bind(port as u16);
}