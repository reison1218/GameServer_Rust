use super::*;

pub struct PacketDes {
    cmd: u32,
    user_id: Option<u32>,
    is_broad: bool,
    is_client: bool,
}

pub struct Packet {
    packet_des: PacketDes,
    bytes: Vec<u8>,
}

impl PacketDes {
    pub fn new(cmd: u32) -> PacketDes {
        PacketDes {
            cmd: cmd,
            user_id: None,
            is_broad: false,
            is_client: true,
        }
    }
}

impl Packet {
    pub fn new(packet_des: PacketDes) -> Packet {
        let v: Vec<u8> = Vec::new();
        Packet {
            packet_des: packet_des,
            bytes: v,
        }
    }

    pub fn get_cmd(&self) -> u32 {
        self.packet_des.cmd
    }

    pub fn set_bytes(&mut self, bytes: &[u8]) {
        self.bytes = Vec::from(bytes);
    }

    pub fn borrow_des(&self) -> Option<&PacketDes> {
        Some(&self.packet_des)
    }

    pub fn get_data(&self) -> &[u8] {
        &self.bytes[..]
    }

    pub fn set_user_id(&mut self, user_id: u32) {
        self.packet_des.user_id = Some(user_id);
    }

    pub fn get_user_id(&self) -> Option<u32> {
        self.packet_des.user_id
    }
}
