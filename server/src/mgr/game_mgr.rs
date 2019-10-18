use super::*;

pub struct GameMgr {
    pub players: HashMap<u32, User>,
    pub pool: DbPool,
    pub channels: ChannelMgr,
    pub cmdMap: HashMap<u32, fn(&mut GameMgr, &Packet), RandomState>,
}

impl GameMgr {
    pub fn new(pool: DbPool) -> GameMgr {
        let mut players: HashMap<u32, User> = HashMap::new();
        let mut channels = ChannelMgr::new();
        let mut gm = GameMgr {
            players: players,
            pool: pool,
            channels: channels,
            cmdMap: HashMap::new(),
        };
        gm
    }

    pub fn invok(&mut self, packet: Packet) {}

    fn cmd_init(&mut self, packet: &Packet) {
        self.cmdMap.insert(123, logOff);
        self.cmdMap.insert(123, sync);
    }
}

fn logOff(gm: &mut GameMgr, packet: &Packet) {}

fn sync(gm: &mut GameMgr, packet: &Packet) {}
