use crossbeam::channel::Sender;
use log::warn;
use protobuf::Message;
use std::collections::HashMap;
use tools::cmd_code::{BattleCode, ServerCommonCode};
use tools::protos::server_protocol::R_B_START;
use tools::tcp::TcpSender;
use tools::util::packet::Packet;

#[derive(Default)]
pub struct GameCenterMgr {
    pub room_center: Option<Sender<Vec<u8>>>,         //房间中心
    pub gate_clients: HashMap<usize, GateClient>,     //gate路由服客户端
    pub battle_clients: HashMap<usize, BattleClient>, //战斗服客户端
    pub user_w_gate: HashMap<u32, usize>,             //玩家对应gate
    pub user_w_battle: HashMap<u32, usize>,           //玩家对应战斗服
}

impl GameCenterMgr {
    pub fn new() -> Self {
        GameCenterMgr::default()
    }

    pub fn handler(&mut self, packet: &Packet) {
        let cmd = packet.get_cmd();
        //开始战斗,负载均衡，分配战斗服务器
        if cmd == BattleCode::Start.into_u32() {
            self.slb(packet.clone());
        }
    }

    ///负载均衡
    pub fn slb(&mut self, packet: Packet) {
        let mut proto = R_B_START::new();
        let res = proto.merge_from_bytes(packet.get_data());
        if let Err(e) = res {
            warn!("{:?}", e);
            return;
        }

        //找出房间数最小的那个服，若没有，则默认第一个
        let bc_res = self
            .battle_clients
            .values_mut()
            .min_by(|x, y| x.room_num.cmp(&y.room_num));
        if let None = bc_res {
            warn!("could not find min room num of battle server!");
            return;
        }
        let bc_res = bc_res.unwrap();
        let battle_token = bc_res.sender.token;
        for member in proto.get_room_pt().members.iter() {
            let user_id = member.user_id;
            self.user_w_battle.insert(user_id, battle_token);
        }
    }

    ///玩家离开
    pub fn user_leave(&mut self, cmd: u32, user_id: u32) {
        if cmd == ServerCommonCode::LineOff.into_u32() {
            self.user_w_battle.remove(&user_id);
            self.user_w_gate.remove(&user_id);
        }
    }

    pub fn get_room_center_mut(&mut self) -> &mut Sender<Vec<u8>> {
        self.room_center.as_mut().unwrap()
    }

    pub fn get_gate_client_mut(&mut self, user_id: u32) -> anyhow::Result<&mut GateClient> {
        let res = self.user_w_gate.get(&user_id);
        if let None = res {
            anyhow::bail!("could not find gate's token by user_id:{}!", user_id)
        }
        let token = res.unwrap();
        let res = self.gate_clients.get_mut(token);
        if let None = res {
            anyhow::bail!("could not find GateClient by token:{}!", token)
        }
        let gc = res.unwrap();
        Ok(gc)
    }

    pub fn get_battle_client_mut(&mut self, user_id: u32) -> anyhow::Result<&mut BattleClient> {
        let res = self.user_w_battle.get(&user_id);
        if let None = res {
            anyhow::bail!("could not find battle's token by user_id:{}!", user_id)
        }
        let token = res.unwrap();
        let res = self.battle_clients.get_mut(token);
        if let None = res {
            anyhow::bail!("could not find BattleClient by token:{}!", token)
        }
        let gc = res.unwrap();
        Ok(gc)
    }

    pub fn set_room_sender(&mut self, sender: Sender<Vec<u8>>) {
        self.room_center = Some(sender);
    }

    pub fn add_gate_client(&mut self, sender: TcpSender) {
        let key = sender.token;
        let gc = GateClient::new(sender);
        self.gate_clients.insert(key, gc);
    }

    pub fn add_battle_client(&mut self, sender: TcpSender) {
        let key = sender.token;
        let rc = BattleClient::new(sender);
        self.battle_clients.insert(key, rc);
    }
}

pub struct GateClient {
    sender: TcpSender,
}

impl GateClient {
    pub fn new(sender: TcpSender) -> Self {
        let gc = GateClient { sender };
        gc
    }

    pub fn send(&mut self, bytes: Vec<u8>) {
        self.sender.send(bytes);
    }
}

pub struct BattleClient {
    sender: TcpSender,
    room_num: u32,
}

impl BattleClient {
    pub fn new(sender: TcpSender) -> Self {
        let rc = BattleClient {
            sender,
            room_num: 0,
        };
        rc
    }

    pub fn send(&mut self, bytes: Vec<u8>) {
        self.sender.send(bytes);
    }
}
