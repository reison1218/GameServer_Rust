use tools::tcp_message_io::TcpHandler;

///玩家会话封装结构体
pub struct GateUser {
    tcp: Option<TcpHandler>, //tcp的stream
}

impl GateUser {
    pub fn new(tcp: Option<TcpHandler>) -> Self {
        GateUser { tcp: tcp }
    }

    pub fn close(&self) {
        if self.tcp.is_some() {
            let tcp = self.tcp.as_ref().unwrap();
            let endpoint = tcp.endpoint;
            tcp.node_handler.network().remove(endpoint.resource_id());
        }
    }

    #[warn(dead_code)]
    pub fn get_token(&self) -> usize {
        let mut token = 0_usize;
        if self.tcp.is_some() {
            token = self.tcp.as_ref().unwrap().endpoint.resource_id().raw();
        }
        token
    }

    pub fn get_tcp_mut_ref(&mut self) -> &mut TcpHandler {
        self.tcp.as_mut().unwrap()
    }
}
