use tools::http::HttpServerHandler;

pub struct TestHandler;
impl HttpServerHandler for TestHandler {
    fn get_path(&self) -> &str {
        "/slg/test"
    }
}
