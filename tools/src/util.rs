///bytebuf封装，用户读写字节数组
pub mod bytebuf {
    pub enum ReadError {
        None,
        NotEnough,
        Zero,
    }

    #[derive(Clone, Debug, Default)]
    pub struct ByteBuf {
        bytes: Vec<u8>, //bytes数据
        index: usize,   //读写指针
    }

    impl ByteBuf {
        ///创建一个空到bytebuf结构体
        pub fn new() -> ByteBuf {
            ByteBuf {
                bytes: Vec::new(),
                index: 0,
            }
        }

        pub fn get_len(&self)->usize{
            self.bytes.len()
        }

        ///通过bytes数组创建bytebuf结构体
        pub fn from(bytes: &[u8]) -> ByteBuf {
            let mut bb = ByteBuf {
                bytes: Vec::new(),
                index: 0,
            };
            bb.push_array(bytes);
            bb
        }

        pub fn form_vec(bytes: Vec<u8>) -> ByteBuf {
            let bb = ByteBuf {
                bytes: bytes,
                index: 0,
            };
            bb
        }

        pub fn to_string(&self) -> String {
            let v = self.bytes.clone();
            let s = String::from_utf8(v);
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

        pub fn push_str(&mut self, _str: &str) {
            for i in _str.as_bytes() {
                self.bytes.push(*i);
            }
        }

        pub fn push_u32(&mut self, i: u32) {
            let value: [u8; 4] = i.to_ne_bytes();
            for i in &value {
                self.bytes.push(*i);
            }
        }

        pub fn push_u16(&mut self, i: u16) {
            let value: [u8; 2] = i.to_ne_bytes();
            for i in &value {
                self.bytes.push(*i);
            }
        }

        pub fn push_u64(&mut self, i: u64) {
            let value: [u8; 8] = i.to_ne_bytes();
            for i in &value {
                self.bytes.push(*i);
            }
        }

        ///push char进去
        pub fn push_char(&mut self, c: char) {
            self.bytes.push(c as u8);
        }

        ///push字符串进去
        pub fn push_string(&mut self, s: String) {
            for i in s.as_bytes() {
                self.bytes.push(*i);
            }
        }

        ///读取指定长度的字节数
        pub fn read_bytes_size(&mut self,size:usize) -> Result<&[u8],&str> {
            if self.bytes.len() - self.index < size {
                return Err("could not read u32,the readable u8 array is notEnough!");
            }
            let end= self.index+size;
            let res = &self.bytes[self.index..end];
            self.index+=size;
            Ok(res)
        }

        ///读取4个字节并拼成一个u32
        pub fn read_u32(&mut self) -> Result<u32, &str> {
            if self.bytes.len() - self.index < 4 {
                return Err("could not read u32,the readable u8 array is notEnough!");
            }
            let buf_bytes = &self.bytes[self.index..=self.index + 3];
            self.index += 4;
            let mut bytes = [0; 4];
            bytes.copy_from_slice(buf_bytes);
            Ok(u32::from_ne_bytes(bytes))
        }

        pub fn read_u16(&mut self) -> Result<u16, &str> {
            if self.bytes.len() - self.index < 2 {
                return Err("could not read u16,the readable u8 array is notEnough!");
            }
            let buf_bytes = &self.bytes[self.index..=self.index + 1];
            self.index += 2;
            let mut bytes = [0; 2];
            bytes.copy_from_slice(buf_bytes);
            Ok(u16::from_ne_bytes(bytes))
        }

        pub fn read_u64(&mut self) -> Result<u64, &str> {
            if self.bytes.len() - self.index < 8 {
                return Err("could not read u64,the readable u8 array is notEnough!");
            }
            let buf_bytes = &self.bytes[self.index..=self.index + 7];
            self.index += 8;
            let mut bytes = [0; 8];
            bytes.copy_from_slice(buf_bytes);
            Ok(u64::from_ne_bytes(bytes))
        }

        pub fn read_u8(&mut self) -> Result<u8, &str> {
            if self.bytes.len() - self.index < 1 {
                return Err("could not read u8,the readable u8 array is notEnough!");
            }
            let b = self.bytes.get(self.index).unwrap();
            self.index += 1;
            Ok(*b)
        }

        pub fn read_bytes(&mut self) -> Result<&[u8], &str> {
            let v = &self.bytes[self.index..];
            self.index = self.bytes.len() - 1;
            Ok(v)
        }

        pub fn into_bytes(self) -> Vec<u8> {
            self.bytes
        }
    }
}

///数据包封装，用户封装传输的数据包
pub mod packet {
    use crate::util::bytebuf::ByteBuf;
    #[derive(Debug, Default, Clone)]
    pub struct PacketDes {
        cmd: u32,
        len: u32,
        user_id: u32,
        is_broad: bool,  //是否需要广播
        is_client: bool, //是否需要广播
    }

    #[derive(Debug, Default, Clone)]
    pub struct Packet {
        packet_des: PacketDes,
        bytes: Vec<u8>,
    }

    impl PacketDes {
        pub fn new(cmd: u32, len: u32, user_id: u32) -> PacketDes {
            PacketDes {
                cmd: cmd,
                len: len,
                user_id: user_id,
                is_broad: false,
                is_client: true,
            }
        }
    }

    unsafe impl Send for Packet {}

    unsafe impl Sync for Packet {}

    impl Packet {
        pub fn new(cmd: u32, len: u32, user_id: u32) -> Packet {
            let des = PacketDes::new(cmd, len, user_id);
            let v: Vec<u8> = Vec::new();
            Packet {
                packet_des: des,
                bytes: v,
            }
        }



        pub fn get_user_id(&self) -> u32 {
            self.packet_des.user_id
        }

        pub fn set_user_id(&mut self, user_id: u32) {
            self.packet_des.user_id = user_id;
        }

        pub fn set_is_broad(&mut self, is_broad: bool) {
            self.packet_des.is_broad = is_broad;
        }

        pub fn set_is_client(&mut self, is_client: bool) {
            self.packet_des.is_client = is_client;
        }

        pub fn build_array_from_server(bytes:Vec<u8>)->Result<Vec<Packet>, String>{
            let mut bb = ByteBuf::form_vec(bytes);
            let mut v = Vec::new();
            loop{
                if bb.index() == bb.get_len(){
                    return Ok(v);
                }
                let cmd = bb.read_u32()?;
                let len = bb.read_u32()?;
                let user_id = bb.read_u32()?;
                let is_client = bb.read_u8()? != 0;
                let is_broad = bb.read_u8()? != 0;
                let body_size = len - 14;
                let mut packet = Packet::new(cmd, len, user_id);
                packet.set_user_id(user_id);
                packet.set_cmd(cmd);
                packet.set_is_client(is_client);
                packet.set_is_broad(is_broad);
                if body_size ==0{
                    v.push(packet);
                    return Ok(v);
                }
                packet.set_data(bb.read_bytes_size(body_size as usize)?);
                v.push(packet);
            }
            Ok(v)
        }

        pub fn build_array_from_client(bytes:Vec<u8>)->Result<Vec<Packet>, String>{
            let mut bb = ByteBuf::form_vec(bytes);
            let mut v = Vec::new();
            loop{
                if bb.index() == bb.get_len(){
                    return Ok(v);
                }
                let cmd = bb.read_u32()?;
                let len = bb.read_u32()?;
                bb.read_u32()?;
                bb.read_u32()?;
                let body_size = len - 16;
                let mut packet = Packet::new(cmd, len, 0);
                packet.set_is_client(true);
                packet.set_is_broad(false);
                if body_size>0{
                    packet.set_data(bb.read_bytes_size(body_size as usize)?);
                }
                v.push(packet);
            }
            Ok(v)
        }

        ///bytebuf转换成packet,只能用于服务端进程内部通信！
        pub fn from_only_server(bytes: Vec<u8>) -> Result<Packet, String> {
            let mut bb = ByteBuf::form_vec(bytes);
            let cmd = bb.read_u32()?;
            let len  = bb.read_u32()?;
            let user_id = bb.read_u32()?;
            let is_client = bb.read_u8()? != 0;
            let is_broad = bb.read_u8()? != 0;
            let mut packet = Packet::new(cmd, 0, user_id);
            packet.set_is_client(is_client);
            packet.set_is_broad(is_broad);
            packet.set_data(bb.read_bytes_size(len as usize)?);
            Ok(packet)
        }

        ///bytebuf转换成packet,只能用于服务端进程内部通信！
        pub fn from_only_client(bytes: Vec<u8>) -> Result<Packet, String> {
            let mut bb = ByteBuf::form_vec(bytes);
            let cmd = bb.read_u32()?;
            let len = bb.read_u32()?;
            bb.read_u32()?;
            bb.read_u32()?;
            let mut packet = Packet::new(cmd, len, 0);
            packet.set_is_client(true);
            packet.set_is_broad(false);
            packet.set_data(bb.read_bytes()?);
            Ok(packet)
        }

        pub fn all_to_client_vec(&self) -> Vec<u8> {
            let mut bb = ByteBuf::new();
            bb.push_u32(self.get_cmd());
            bb.push_u32(self.get_len());
            bb.push_u32(0);
            bb.push_u32(0);
            bb.push_array(self.get_data());
            bb.into_bytes()
        }

        ///获得命令
        pub fn get_cmd(&self) -> u32 {
            self.packet_des.cmd
        }

        //设置命令
        pub fn set_cmd(&mut self, cmd: u32) {
            self.packet_des.cmd = cmd;
        }

        ///设置body
        pub fn set_data(&mut self, bytes: &[u8]) {
            self.bytes = Vec::from(bytes);
        }

        ///设置body
        pub fn set_data_from_vec(&mut self, v: Vec<u8>) {
            self.bytes = v;
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
        pub fn get_data_vec(self) -> Vec<u8> {
            self.bytes
        }

        pub fn set_len(&mut self, len: u32) {
            self.packet_des.len = len;
        }
        ///获得userid
        pub fn get_len(&self) -> u32 {
            self.packet_des.len
        }

        pub fn cal_len(&mut self) {
            let len = 16 + self.get_data().len();
            self.set_len(len as u32);
        }

        ///转换成bytebuf
        pub fn to_client_bytebuf(&mut self) -> ByteBuf {
            self.cal_len();
            let mut bb = ByteBuf::new();
            bb.push_u32(self.get_cmd());
            bb.push_u32(self.get_len());
            bb.push_u32(0);
            bb.push_u32(0);
            bb.push_array(self.get_data());
            bb
        }

        pub fn is_client(&self) -> bool {
            self.packet_des.is_client
        }

        pub fn is_broad(&self) -> bool {
            self.packet_des.is_broad
        }

        ///转换成bytebuf
        pub fn to_server_bytebuf(&self) -> ByteBuf {
            let mut bb = ByteBuf::new();
            bb.push_u32(self.get_cmd());
            bb.push_u32(14+self.get_data().len() as u32);
            bb.push_u32(self.get_user_id());
            bb.push(self.is_client() as u8);
            bb.push(self.is_broad() as u8);
            bb.push_array(self.get_data());
            bb
        }

        ///转换成byte数组
        pub fn build_client_bytes(&mut self) -> Vec<u8> {
            let byte_buf = self.to_client_bytebuf();
            byte_buf.into_bytes()
        }

        ///转换成byte数组
        pub fn build_server_bytes(&self) -> Vec<u8> {
            let byte_buf = self.to_server_bytebuf();
            byte_buf.into_bytes()
        }

        ///构建一个用于通信返回的bytes数组
        pub fn build_packet_bytes(cmd:u32,user_id:u32,data:Vec<u8>,is_server:bool,is_2_client:bool)->Vec<u8>{
            let mut packet = Packet::new(cmd,(16+data.len()) as u32,user_id);
            packet.set_data_from_vec(data);
            packet.packet_des.is_client = is_2_client;
            if is_server {
                packet.build_server_bytes()
            }else {
                packet.build_client_bytes()
            }
        }
    }
}
