use super::*;

pub enum ClientType{
    GateServer = 1,
    RoomServer = 2,
}

///gateserver客户端封装
#[derive(Default, Clone)]
pub struct NetClient {
    client_type:u8,          //客户端类型
    id: u32,                //gateserver id
    address: String,        //gateserver 地址
    tcp: Option<TcpSender>, //tcp的stream
}

impl NetClient {
    pub fn new(client_type:u8,id: u32, address: String, tcp: Option<TcpSender>) -> Self {
        NetClient { client_type,id, address, tcp }
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
