use super::*;
pub struct GateMgr{
    pub players: HashMap<u32, GateUser>,
}

impl GateMgr {
    pub fn new() -> GateMgr {
        let mut players: HashMap<u32, GateUser> = HashMap::new();
        GateMgr {
            players: players,
        }
    }

    pub fn invok(&mut self, packet: Packet) {}
}