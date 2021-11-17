use tools::net_message_io::NetHandler;

///玩家会话封装结构体
pub struct GateUser {
    net: Option<NetHandler>, //tcp的stream
}

impl GateUser {
    pub fn new(tcp: Option<NetHandler>) -> Self {
        GateUser { net: tcp }
    }

    pub fn close(&self) {
        if self.net.is_some() {
            let tcp = self.net.as_ref().unwrap();
            let endpoint = tcp.endpoint;
            tcp.node_handler.network().remove(endpoint.resource_id());
        }
    }

    #[warn(dead_code)]
    pub fn get_token(&self) -> usize {
        let mut token = 0_usize;
        if self.net.is_some() {
            token = self.net.as_ref().unwrap().endpoint.resource_id().raw();
        }
        token
    }

    pub fn get_net_mut_ref(&mut self) -> &mut NetHandler {
        self.net.as_mut().unwrap()
    }
}
