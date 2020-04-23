///将两个u32合成一个u64
pub fn  combineInt2Long( low:u32, high:u32)->u64{
    let a = low as u64 & (0xFFFFFFFF as u64);
    let b = ((high as u64) << 32) & 0xFFFFFFFF00000000 as u64;
    return a|b;
}

///将一个u64拆成两个u32
pub fn  separateLong2int(val:u64)->[u32;2] {
    let mut ret:[u32;2] = [0;2];
    ret[0] = (0xFFFFFFFF & val) as u32;
    ret[1] =  ((0xFFFFFFFF00000000 & val) >> 32) as u32;
    return ret;
}