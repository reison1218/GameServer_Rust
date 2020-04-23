

///bytebuf封装，用户读写字节数组
pub mod bytebuf{
    use std::mem::transmute;

    pub enum ReadError{
        None,
        NotEnough,
        Zero
    }
    
    #[derive(Clone)]
    pub struct ByteBuf {
        bytes: Vec<u8>,//bytes数据
        index: usize,//读写指针
    }

    impl ByteBuf {
        ///创建一个空到bytebuf结构体
        pub fn new() -> ByteBuf {
            ByteBuf {
                bytes: Vec::new(),
                index: 0,
            }
        }

        ///通过bytes数组创建bytebuf结构体
        pub fn from(bytes:&[u8])->ByteBuf{
            let mut bb = ByteBuf {
                bytes: Vec::new(),
                index: 0,
            };
            bb.push_array(bytes);
            bb
        }

        pub fn form_vec(bytes:Vec<u8>)->ByteBuf{
            let bb = ByteBuf {
                bytes: bytes,
                index: 0,
            };
            bb
        }

        pub fn to_string(&self)->String{
            let v = self.bytes.clone();
            let  s = String::from_utf8(v);
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
                let byte = transmute::<u32, [u8; 4]>(i);

                for i in &byte {
                    self.bytes.push(*i);
                }
            }
        }

        pub fn push_u16(&mut self, i: u16) {
            unsafe {
                let byte = transmute::<u16, [u8; 2]>(i);

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
            }
        }
    }


    unsafe  impl Send for Packet{

    }

    unsafe  impl Sync for Packet{

    }

    impl Packet {
        pub fn new(cmd: u32) -> Packet {
            let mut des = PacketDes::new(cmd);
            des.user_id = None;
            let v: Vec<u8> = Vec::new();
            Packet {
                packet_des: des,
                bytes: v,
            }
        }

        ///bytebuf转换成packet
        pub fn from(mut bb: ByteBuf) -> Packet {
            let cmd = bb.read_u32().unwrap();
            let user_id = bb.read_u32().unwrap();
            let mut packet = Packet::new(cmd);
            packet.set_user_id(user_id);
            packet.set_cmd(cmd);
            packet.set_data(bb.read_bytes().unwrap());
            packet
        }

        pub fn all_to_vec(&self)->Vec<u8>{
            let mut v = Vec::new();
            let mut bb = ByteBuf::new();
            let mut user_id = 0 as u32;
            if self.get_user_id().is_some(){
                user_id = self.get_user_id().unwrap();
            }
            bb.push_u32(self.get_cmd());
            bb.push_u32(user_id);
            bb.push_array(self.get_data());
            for i in bb.bytes(){
                v.push(*i);
            }
            v
        }

        ///获得命令
        pub fn get_cmd(&self) -> u32 {
            self.packet_des.cmd
        }

        //设置命令
        pub fn set_cmd(&mut self,cmd:u32){
            self.packet_des.cmd = cmd;
        }

        ///设置body
        pub fn set_data(&mut self, bytes: &[u8]) {
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

        ///获得body,慎重使用，此函数直接转让bytes所有权！
        pub fn get_data_vec(self)->Vec<u8>{
           self.bytes
        }

        ///设置userid
        pub fn set_user_id(&mut self, user_id: u32) {
            self.packet_des.user_id = Some(user_id);
        }

        ///获得userid
        pub fn get_user_id(&self) -> Option<u32> {
            self.packet_des.user_id
        }

        ///转换成byte数组
        pub fn to_bytebuf(&self)->ByteBuf{
            let mut bb = ByteBuf::new();
            bb.push_u32(self.get_cmd());
            if self.get_user_id().is_some(){
                bb.push_u32(self.get_user_id().unwrap());
            }else{
                bb.push_u32(0);
            }

            bb.push_array(self.get_data());
            bb
        }
    }
}