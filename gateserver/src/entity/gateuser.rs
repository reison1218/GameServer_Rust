use std::sync::Arc;
use tools::tcp::TcpSender;
use ws::CloseCode;
use ws::Sender as WsSender;

///玩家会话封装结构体
pub struct GateUser {
    ws: Option<Arc<WsSender>>, //websocket会话封装
    tcp: Option<TcpSender>,    //tcp的stream
}

impl GateUser {
    pub fn new(ws: Option<Arc<WsSender>>, tcp: Option<TcpSender>) -> Self {
        GateUser { ws: ws, tcp: tcp }
    }

    pub fn close(&mut self) {
        if self.ws.is_some() {
            self.get_ws_ref().close(CloseCode::Invalid).unwrap();
        }
        if self.tcp.is_some() {
            self.tcp.as_mut().unwrap().send(vec![]);
        }
    }

    #[warn(dead_code)]
    pub fn get_token(&self) -> usize {
        let mut token = 0_usize;
        if self.tcp.is_some() {
            token = self.tcp.as_ref().unwrap().token
        } else if self.ws.is_some() {
            token = self.ws.as_ref().unwrap().token().0
        }
        token
    }

    pub fn get_ws_ref(&self) -> &Arc<WsSender> {
        self.ws.as_ref().unwrap()
    }

    #[warn(dead_code)]
    pub fn get_ws_mut_ref(&mut self) -> &mut Arc<WsSender> {
        self.ws.as_mut().unwrap()
    }

    pub fn get_tcp_mut_ref(&mut self) -> &mut TcpSender {
        self.tcp.as_mut().unwrap()
    }
}
