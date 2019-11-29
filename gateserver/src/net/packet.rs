use super::*;
use crate::net::bytebuf::ByteBuf;

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

    ///bytebuf转换成packet
    pub fn from(bb: ByteBuf) -> Packet {
        let mut bb = bb;
        let mut pd = PacketDes::new(bb.read_u32().unwrap());
        pd.user_id = Some(bb.read_u32().unwrap());
        pd.is_broad = false;
        pd.is_client = true;
        let mut packet = Packet::new(pd);
        packet.set_bytes(bb.read_bytes().unwrap());
        packet
    }

    pub fn is_client(&self) -> bool {
        self.packet_des.is_client
    }

    ///获得命令
    pub fn get_cmd(&self) -> u32 {
        self.packet_des.cmd
    }

    ///设置body
    pub fn set_bytes(&mut self, bytes: &[u8]) {
        self.bytes = Vec::from(bytes);
    }

    ///来一发des的借用
    pub fn borrow_des(&self) -> Option<&PacketDes> {
        Some(&self.packet_des)
    }

    ///获得body
    pub fn get_data(&self) -> &[u8] {
        &self.bytes[..]
    }

    ///设置userid
    pub fn set_user_id(&mut self, user_id: u32) {
        self.packet_des.user_id = Some(user_id);
    }

    ///获得userid
    pub fn get_user_id(&self) -> Option<u32> {
        self.packet_des.user_id
    }
}
