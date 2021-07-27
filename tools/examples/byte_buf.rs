use tools::util::bytebuf::ByteBuf;

pub fn main() {
    let mut bb = ByteBuf::new();
    //push
    bb.push(1);
    bb.push(8);
    bb.push_u16(16);
    bb.push_u32(32);
    bb.push_u64(64);
    bb.push_str("hello");
    bb.push_string("world".to_owned());
    //read
    println!("{:?}", bb);
    println!("{}", bb.read_u8().unwrap());
    println!("{}", bb.read_u8().unwrap());
    println!("{}", bb.read_u16().unwrap());
    println!("{}", bb.read_u32().unwrap());
    println!("{}", bb.read_u64().unwrap());
    let res = bb.read_bytes_size(5).unwrap();
    println!("{:?}", String::from_utf8(res.to_vec()));
    let res = bb.read_bytes_size(5).unwrap();
    println!("{:?}", String::from_utf8(res.to_vec()));
}
