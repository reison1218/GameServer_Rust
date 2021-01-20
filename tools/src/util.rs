use chrono::Timelike;

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

    impl From<Vec<u8>> for ByteBuf {
        fn from(v: Vec<u8>) -> Self {
            let bb = ByteBuf { bytes: v, index: 0 };
            bb
        }
    }

    impl From<&[u8]> for ByteBuf {
        fn from(v: &[u8]) -> Self {
            let mut bb = ByteBuf::default();
            bb.bytes.extend_from_slice(v);
            bb
        }
    }

    impl ByteBuf {
        ///创建一个空到bytebuf结构体
        pub fn new() -> ByteBuf {
            ByteBuf {
                bytes: Vec::new(),
                index: 0,
            }
        }

        pub fn get_len(&self) -> usize {
            self.bytes.len()
        }

        pub fn to_string(&self) -> anyhow::Result<String> {
            let v = self.bytes.clone();
            let s = String::from_utf8(v)?;
            Ok(s)
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
            self.bytes.extend_from_slice(bytes);
        }

        pub fn push_str(&mut self, _str: &str) {
            self.bytes.extend_from_slice(_str.as_bytes());
        }

        pub fn push_u32(&mut self, i: u32) {
            let value: [u8; 4] = i.to_ne_bytes();
            self.bytes.extend_from_slice(value.as_ref());
        }

        pub fn push_u16(&mut self, i: u16) {
            let value: [u8; 2] = i.to_ne_bytes();
            self.bytes.extend_from_slice(value.as_ref());
        }

        pub fn push_u64(&mut self, i: u64) {
            let value: [u8; 8] = i.to_ne_bytes();
            self.bytes.extend_from_slice(value.as_ref());
        }

        ///push char进去
        pub fn push_char(&mut self, c: char) {
            self.bytes.push(c as u8);
        }

        ///push字符串进去
        pub fn push_string(&mut self, s: String) {
            self.bytes.extend_from_slice(s.as_bytes());
        }

        ///读取指定长度的字节数
        pub fn read_bytes_size(&mut self, size: usize) -> anyhow::Result<&[u8]> {
            if self.bytes.len() - self.index < size {
                anyhow::bail!("could not read u32,the readable u8 array is notEnough!")
            }
            let end = self.index + size;
            let res = &self.bytes[self.index..end];
            self.index += size;
            Ok(res)
        }

        ///读取4个字节并拼成一个u32
        pub fn read_u32(&mut self) -> anyhow::Result<u32> {
            if self.bytes.len() - self.index < 4 {
                anyhow::bail!("could not read u32,the readable u8 array is notEnough!")
            }
            let buf_bytes = &self.bytes[self.index..=self.index + 3];
            self.index += 4;
            let mut bytes = [0; 4];
            bytes.copy_from_slice(buf_bytes);
            Ok(u32::from_ne_bytes(bytes))
        }

        ///读取两个字节并拼成一个u16
        pub fn read_u16(&mut self) -> anyhow::Result<u16> {
            if self.bytes.len() - self.index < 2 {
                anyhow::bail!("could not read u16,the readable u8 array is notEnough!")
            }
            let buf_bytes = &self.bytes[self.index..=self.index + 1];
            self.index += 2;
            let mut bytes = [0; 2];
            bytes.copy_from_slice(buf_bytes);
            Ok(u16::from_ne_bytes(bytes))
        }

        ///读取8个字节并拼成一个u64
        pub fn read_u64(&mut self) -> anyhow::Result<u64> {
            if self.bytes.len() - self.index < 8 {
                anyhow::bail!("could not read u64,the readable u8 array is notEnough!")
            }
            let buf_bytes = &self.bytes[self.index..=self.index + 7];
            self.index += 8;
            let mut bytes = [0; 8];
            bytes.copy_from_slice(buf_bytes);
            Ok(u64::from_ne_bytes(bytes))
        }

        ///读取1个字节并拼成一个u8
        pub fn read_u8(&mut self) -> anyhow::Result<u8> {
            if self.bytes.len() - self.index < 1 {
                anyhow::bail!("could not read u8,the readable u8 array is notEnough!")
            }
            let b = self.bytes.get(self.index).unwrap();
            self.index += 1;
            Ok(*b)
        }

        ///读取所有字节
        pub fn read_bytes(&mut self) -> &[u8] {
            let v = &self.bytes[self.index..];
            self.index = self.bytes.len() - 1;
            v
        }

        ///获得所有字节
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
        is_broad: bool,    //是否需要广播
        is_client: bool,   //是否客户端
        server_token: u32, //服务器对应的token
    }

    #[derive(Debug, Default, Clone)]
    pub struct Packet {
        packet_des: PacketDes,
        bytes: Vec<u8>,
    }

    impl PacketDes {
        pub fn new(cmd: u32, len: u32, user_id: u32) -> PacketDes {
            PacketDes {
                cmd,
                len,
                user_id,
                is_broad: false,
                is_client: true,
                server_token: 0,
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

        pub fn set_server_token(&mut self, server_token: u32) {
            self.packet_des.server_token = server_token;
        }

        ///获得server token
        pub fn get_server_token(&self) -> u32 {
            self.packet_des.server_token
        }

        pub fn set_is_client(&mut self, is_client: bool) {
            self.packet_des.is_client = is_client;
        }

        ///解析tcp流数据，可能饱含多个数据包，循环解析
        pub fn build_array_from_server(bytes: Vec<u8>) -> anyhow::Result<Vec<Packet>> {
            let mut bb = ByteBuf::from(bytes);
            let mut v = Vec::new();
            loop {
                if bb.index() == bb.get_len() {
                    return Ok(v);
                }
                let cmd = bb.read_u32()?;
                let len = bb.read_u32()?;
                let user_id = bb.read_u32()?;
                let is_client = bb.read_u8()? != 0;
                let is_broad = bb.read_u8()? != 0;
                let server_token = bb.read_u32()?;
                let body_size = len - 18;
                let mut packet = Packet::new(cmd, len, user_id);
                packet.set_user_id(user_id);
                packet.set_cmd(cmd);
                packet.set_is_client(is_client);
                packet.set_is_broad(is_broad);
                packet.set_server_token(server_token);
                if body_size > 0 {
                    packet.set_data(bb.read_bytes_size(body_size as usize)?);
                }
                v.push(packet);
            }
        }

        pub fn build_array_from_client(bytes: Vec<u8>) -> anyhow::Result<Vec<Packet>> {
            let mut bb = ByteBuf::from(bytes);
            let mut v = Vec::new();
            loop {
                if bb.index() == bb.get_len() {
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
                if body_size > 0 {
                    packet.set_data(bb.read_bytes_size(body_size as usize)?);
                }
                v.push(packet);
            }
        }

        ///bytebuf转换成packet,只能用于服务端进程内部通信！
        pub fn from_only_server(bytes: Vec<u8>) -> anyhow::Result<Packet> {
            let mut bb = ByteBuf::from(bytes);
            let cmd = bb.read_u32()?;
            let len = bb.read_u32()?;
            let user_id = bb.read_u32()?;
            let is_client = bb.read_u8()? != 0;
            let is_broad = bb.read_u8()? != 0;
            let server_token = bb.read_u32()?;
            let mut packet = Packet::new(cmd, 0, user_id);
            packet.set_is_client(is_client);
            packet.set_is_broad(is_broad);
            packet.set_server_token(server_token);
            packet.set_data(bb.read_bytes_size(len as usize)?);
            Ok(packet)
        }

        ///bytebuf转换成packet,只能用于服务端进程内部通信！
        pub fn from_only_client(bytes: Vec<u8>) -> anyhow::Result<Packet> {
            let mut bb = ByteBuf::from(bytes);
            let cmd = bb.read_u32()?;
            let len = bb.read_u32()?;
            bb.read_u32()?;
            bb.read_u32()?;
            let mut packet = Packet::new(cmd, len, 0);
            packet.set_is_client(true);
            packet.set_is_broad(false);
            packet.set_data(bb.read_bytes());
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
            bb.push_u32(18 + self.get_data().len() as u32);
            bb.push_u32(self.get_user_id());
            bb.push(self.is_client() as u8);
            bb.push(self.is_broad() as u8);
            bb.push_u32(self.get_server_token());
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
        pub fn build_packet_bytes(
            cmd: u32,
            user_id: u32,
            data: Vec<u8>,
            is_server: bool,
            is_2_client: bool,
        ) -> Vec<u8> {
            let mut packet = Packet::new(cmd, 0, user_id);
            packet.set_data_from_vec(data);
            packet.packet_des.is_client = is_2_client;
            if is_server {
                packet.build_server_bytes()
            } else {
                packet.build_client_bytes()
            }
        }

        ///构建一个用于通信返回的bytes数组
        pub fn build_packet_bytes_direction(
            cmd: u32,
            user_id: u32,
            data: Vec<u8>,
            is_server: bool,
            is_2_client: bool,
            server_token: u32,
        ) -> Vec<u8> {
            let mut packet = Packet::new(cmd, 0, user_id);
            packet.set_data_from_vec(data);
            packet.set_server_token(server_token);
            packet.packet_des.is_client = is_2_client;
            if is_server {
                packet.build_server_bytes()
            } else {
                packet.build_client_bytes()
            }
        }

        ///构建一个用于通信返回的bytes数组
        pub fn build_push_packet_bytes(
            cmd: u32,
            user_id: u32,
            data: Vec<u8>,
            is_server: bool,
            is_2_client: bool,
        ) -> Vec<u8> {
            let mut packet = Packet::new(cmd, 0, user_id);
            packet.set_data_from_vec(data);
            packet.packet_des.is_client = is_2_client;
            packet.set_is_broad(true);
            if is_server {
                packet.build_server_bytes()
            } else {
                packet.build_client_bytes()
            }
        }
    }
}

///判断给定时间是不是今天
pub fn is_today(time: i64) -> bool {
    let mut today = chrono::Local::now();
    today = today.with_hour(0).unwrap();
    today = today.with_minute(0).unwrap();
    today = today.with_second(0).unwrap();
    today = today.with_second(0).unwrap();
    let today = today.timestamp_millis();
    let next_day = today + 86400 * 1000;
    if time >= today && time <= next_day {
        return true;
    }
    false
}
