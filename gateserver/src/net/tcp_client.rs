use super::*;

pub struct TcpClientHandler {
    ts: Option<TcpStream>,
    cp: Arc<RwLock<ChannelMgr>>,
}

impl TcpClientHandler {
    pub fn new(cp: Arc<RwLock<ChannelMgr>>) -> TcpClientHandler {
        let mut tch = TcpClientHandler { ts: None, cp: cp };
        tch
    }
}

impl ClientHandler for TcpClientHandler {
    fn on_open(&mut self, ts: TcpStream) {
        self.cp.write().unwrap().game_client_channel = Some(ts.try_clone().unwrap());
        self.ts = Some(ts);
    }

    fn on_close(&mut self) {
        let address = CONF_MAP.get_str("gamePort");
        self.on_read(address.to_string());
    }

    fn on_message(&mut self, mess: Vec<u8>) {
        let mut bb = ByteBuf::form_vec(mess);
        let mut packet = Packet::from(bb);

        //判断是否是发给客户端消息
        if packet.is_client() && packet.get_cmd() > 0 {
            info!("属于需要发给客户端的消息！");
            let mut write = self.cp.write().unwrap();
            let mut gate_user = write.get_mut_user_channel_channel(&packet.get_user_id().unwrap());
            if gate_user.is_none() {
                error!("user data is null,id:{}", &packet.get_user_id().unwrap());
                return;
            }
            //            let mut mp = MessPacketPt::new();
            //            mp.cmd = packet.get_cmd();
            //            mp.set_data(packet.get_data_vec());
            //            let bytes = mp.write_to_bytes();
            //            gate_user.unwrap().get_ws_ref().send(&bytes.unwrap()[..]);
            let mut mp = MessPacketPt::new();
            mp.cmd = packet.get_cmd();
            mp.set_data(packet.get_data_vec());
            let mut p = Packet::new(5003);
            p.set_data(&mp.write_to_bytes().unwrap()[..]);
            gate_user.unwrap().get_tcp_mut_ref().sender.send(p);
            info!("回客户端消息");
        } else { //判断是否要转发到其他服务器进程消息
        }
    }

    fn get_address(&self) -> &str {
        let address = CONF_MAP.get_str("gamePort");
        address
    }
}
