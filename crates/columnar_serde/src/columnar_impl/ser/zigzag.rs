pub(crate) fn zig_zag_i16(n: i16) -> u16 {
    ((n << 1) ^ (n >> 15)) as u16
}

pub(crate) fn zig_zag_i32(n: i32) -> u32 {
    ((n << 1) ^ (n >> 31)) as u32
}

pub(crate) fn zig_zag_i64(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

pub(crate) fn zig_zag_i128(n: i128) -> u128 {
    ((n << 1) ^ (n >> 127)) as u128
}