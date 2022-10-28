#[doc(hidden)]
pub(crate) const CONTINUATION_BIT: u8 = 1 << 7;

#[inline]
pub(crate) fn low_bits_of_byte(byte: u8) -> u8 {
    byte & !CONTINUATION_BIT
}

#[inline]
pub(crate) fn low_bits_of_u64(val: u64) -> u8 {
    let byte = val & (std::u8::MAX as u64);
    low_bits_of_byte(byte as u8)
}

#[inline]
pub(crate) fn uleb64(mut val: u64) -> Vec<u8>{
    let mut buf = Vec::new();
    loop {
        let mut byte = low_bits_of_u64(val);
        val >>= 7;
        if val != 0 {
            // More bytes to come, so set the continuation bit.
            byte |= CONTINUATION_BIT;
        }

        buf.push(byte);
        if val == 0 {
            return buf;
        }
    }
}