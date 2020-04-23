

///bytebuf封装，用户读写字节数组
pub mod bytebuf{
    use super::*;
    use std::mem::transmute;

    pub enum ReadError{
        None,
        NotEnough,
        Zero
    }
    
    #[derive(Clone)]
    pub struct ByteBuf {
        bytes: Vec<u8>,
        index: usize,
    }

    impl ByteBuf {
        pub fn new() -> ByteBuf {
            ByteBuf {
                bytes: Vec::new(),
                index: 0,
            }
        }

        pub fn to_string(&self)->String{
            let v = self.bytes.clone();
            let mut s = String::from_utf8(v);
            s.unwrap()
        }

        pub fn index(&self) -> usize {
            self.index
        }

        pub fn set_index(&mut self, index: usize) -> usize {
            self.index = index;
            self.index
        }

        pub fn bytes(&self) -> &[u8] {
            &self.bytes[..]
        }



        pub fn push(&mut self, byte: u8) {
            self.bytes.push(byte);
        }

        pub fn push_array(&mut self, bytes: &[u8]) {
            for i in bytes {
                self.bytes.push(*i);
            }
        }

        pub fn push_str(&mut self, _str: String) {
            for i in _str.as_bytes() {
                self.bytes.push(*i);
            }
        }

        pub fn push_u32(&mut self, i: u32) {
            unsafe {
                let mut byte = transmute::<u32, [u8; 4]>(i);

                for i in &byte {
                    self.bytes.push(*i);
                }
            }
        }

        pub fn push_u16(&mut self, i: u16) {
            unsafe {
                let mut byte = transmute::<u16, [u8; 2]>(i);

                for i in &byte {
                    self.bytes.push(*i);
                }
            }
        }

        pub fn push_u64(&mut self, i: u64) {
            unsafe {
                let mut byte = transmute::<u64, [u8; 8]>(i);

                for i in &byte {
                    self.bytes.push(*i);
                }
            }
        }

        pub fn push_char(&mut self, c: char) {
            self.bytes.push(c as u8);
        }

        pub fn push_string(&mut self, s: String) {


            for i in s.as_bytes(){
                self.bytes.push(*i);
            }

        }

        pub fn read_u32(&mut self) -> Result<u32,&str> {
            if self.bytes.len()-self.index<4{
                return Err("NotEnough");
            }
            let b = &self.bytes[self.index..=self.index + 3];
            self.index += 4;
            let mut int = 0;
            unsafe {
                let mut byte: [u8; 4] = [0; 4];

                for i in 0..3 {
                    byte[i] = b[i];
                }
                int = transmute::<[u8; 4], u32>(byte);
            }
            Ok(int)
        }

        pub fn read_u16(&mut self) -> Result<u16,&str> {
            if self.bytes.len()-self.index<2{
                return Err("NotEnough");
            }

            let b = &self.bytes[self.index..=self.index + 1];
            self.index += 2;
            let mut short = 0;
            unsafe {
                let mut byte: [u8; 2] = [0; 2];

                for i in 0..1 {
                    byte[i] = b[i];
                }
                short = transmute::<[u8; 2], u16>(byte);
            }
            Ok(short)
        }

        pub fn read_u64(&mut self) -> Result<u64,&str> {
            if self.bytes.len()-self.index<8{
                return Err("NotEnough");
            }

            let b = &self.bytes[self.index..=self.index + 7];
            self.index += 8;
            let mut long = 0;
            unsafe {
                let mut byte: [u8; 8] = [0; 8];

                for i in 0..7 {
                    byte[i] = b[i];
                }
                long = transmute::<[u8; 8], u64>(byte);
            }
            Ok(long)
        }

        pub fn read_u8(&mut self) -> Result<u8,&str> {
            if self.bytes.len()-self.index<1{
                return Err("NotEnough");
            }

            let b = self.bytes.get(self.index).unwrap();
            self.index+=1;
            Ok(*b)
        }

        pub fn read_bytes(&mut self) -> Result<&[u8],&str> {
            let  v = &self.bytes[self.index..];
            self.index = self.bytes.len()-1;
            Ok(v)
        }
    }
}

///数据包封装，用户封装传输的数据包
pub mod packet{
    use crate::util::bytebuf::ByteBuf;
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


    unsafe  impl Send for Packet{

    }

    unsafe  impl Sync for Packet{

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



}



