use super::*;

pub struct GameMgr {
    pub players: HashMap<u32, User>,
    pub pool: DbPool,
    pub channels: HashMap<u32, Channel>,
}

impl GameMgr {
    pub fn new(pool: DbPool) -> GameMgr {
        let mut players: HashMap<u32, User> = HashMap::new();
        let mut channels: HashMap<u32, Channel> = HashMap::new();
        GameMgr {
            players: players,
            pool: pool,
            channels: channels,
        }
    }

    pub fn invok(&mut self, packet: Packet) {}
}
