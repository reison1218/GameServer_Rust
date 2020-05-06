use super::*;

///gateserver客户端封装
#[derive(Default, Debug, Clone)]
pub struct GateWay {
    id: u32,                //gateserver id
    address: String,        //gateserver 地址
    tcp: Option<TcpSender>, //tcp的stream
}

impl GateWay {
    pub fn new(id: u32, address: String, tcp: Option<TcpSender>) -> Self {
        GateUser { id, address, tcp }
    }

    pub fn close(&self) {
        if self.tcp.is_some() {}
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_tcp_ref(&self) -> &TcpSender {
        self.tcp.as_ref().unwrap()
    }

    pub fn get_tcp_mut_ref(&mut self) -> &mut TcpSender {
        self.tcp.as_mut().unwrap()
    }
}
