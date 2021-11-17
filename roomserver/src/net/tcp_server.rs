use crate::Lock;
use async_std::task;
use async_trait::async_trait;
use log::{error, info};
use tools::net_message_io::NetHandler;
use tools::net_message_io::TransportWay;
use tools::util::packet::Packet;

///处理客户端所有请求,每个客户端单独分配一个handler
#[derive(Clone)]
pub struct TcpServerHandler {
    pub rm: Lock,
}

unsafe impl Send for TcpServerHandler {}

unsafe impl Sync for TcpServerHandler {}

#[async_trait]
impl tools::net_message_io::MessageHandler for TcpServerHandler {
    async fn try_clone(&self) -> Self {
        self.clone()
    }

    ///客户端tcp链接激活事件
    async fn on_open(&mut self, sender: NetHandler) {
        self.rm.lock().await.set_net_handler(sender);
    }

    ///客户端tcp链接关闭事件
    async fn on_close(&mut self) {
        info!("与tcp客户端断开连接");
    }

    ///客户端读取事件
    async fn on_message(&mut self, mess: &[u8]) {
        let packet_array = Packet::build_array_from_server(mess.to_vec());

        if let Err(e) = packet_array {
            error!("{:?}", e);
            return;
        }
        let packet_array = packet_array.unwrap();

        for packet in packet_array {
            //异步处理业务逻辑
            task::spawn(handler_mess_s(self.rm.clone(), packet));
        }
    }
}

///处理客户端消息
async fn handler_mess_s(rm: Lock, packet: Packet) {
    let mut lock = rm.lock().await;
    lock.invok(packet);
}

///创建新的tcp服务器,如果有问题，终端进程
pub fn new(address: &str, rm: Lock) {
    let sh = TcpServerHandler { rm };
    tools::net_message_io::run(TransportWay::Tcp, address, sh);
}
