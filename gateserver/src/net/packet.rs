use super::*;

pub struct PacketDes {
    pub cmd: u32,
    pub user_id: u32,
    pub is_broad: bool,
}

pub struct Packet {
    pub packet_des: PacketDes,
    pub bytes: Vec<u8>,
}

impl PacketDes {
    pub fn new(cmd: u32, user_id: u32) -> PacketDes {
        PacketDes {
            cmd: cmd,
            user_id: user_id,
            is_broad: false,
        }
    }
}

impl Packet {
    pub fn new(packet_des: PacketDes) -> Packet {
        let mut v: Vec<u8> = Vec::new();
        Packet {
            packet_des: packet_des,
            bytes: v,
        }
    }
}
