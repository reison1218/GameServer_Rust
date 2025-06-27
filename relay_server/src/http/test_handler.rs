use super::*;

pub struct TestHandler;
impl HttpServerHandler for TestHandler {
    fn get_path(&self) -> &str {
        "/gateway/test"
    }
}
