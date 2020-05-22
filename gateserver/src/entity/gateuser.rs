use super::*;
use tools::tcp::{Data, TcpSender};

///玩家会话封装结构体
pub struct GateUser {
    user_id: u32,              //玩家id
    ws: Option<Arc<WsSender>>, //websocket会话封装
    tcp: Option<TcpSender>,    //tcp的stream
}

impl GateUser {
    pub fn new(user_id: u32, ws: Option<Arc<WsSender>>, tcp: Option<TcpSender>) -> Self {
        GateUser {
            user_id: user_id,
            ws: ws,
            tcp: tcp,
        }
    }

    pub fn close(&mut self) {
        if self.ws.is_some() {
            self.get_ws_ref().close(CloseCode::Invalid).unwrap();
        }
        if self.tcp.is_some() {}
    }

    pub fn get_user_id(&self) -> u32 {
        self.user_id
    }

    pub fn get_ws_ref(&self) -> &Arc<WsSender> {
        self.ws.as_ref().unwrap()
    }

    pub fn get_ws_mut_ref(&mut self) -> &mut Arc<WsSender> {
        self.ws.as_mut().unwrap()
    }

    pub fn get_tcp_ref(&self) -> &TcpSender {
        self.tcp.as_ref().unwrap()
    }

    pub fn get_tcp_mut_ref(&mut self) -> &mut TcpSender {
        self.tcp.as_mut().unwrap()
    }
}
