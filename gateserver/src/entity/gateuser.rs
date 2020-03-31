use super::*;
use tcp::tcp::MySyncSender;

///玩家会话封装结构体
pub struct GateUser {
    user_id: u32,              //玩家id
    ws: Option<Arc<WsSender>>, //websocket会话封装
    tcp: Option<MySyncSender>, //tcp的stream
}

impl GateUser {
    pub fn new(user_id: u32, ws: Option<Arc<WsSender>>, tcp: Option<MySyncSender>) -> Self {
        GateUser {
            user_id: user_id,
            ws: ws,
            tcp: tcp,
        }
    }

    pub fn close(&self) {
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

    pub fn get_tcp_ref(&self) -> &MySyncSender {
        self.tcp.as_ref().unwrap()
    }

    pub fn get_tcp_mut_ref(&mut self) -> &mut MySyncSender {
        self.tcp.as_mut().unwrap()
    }
}
