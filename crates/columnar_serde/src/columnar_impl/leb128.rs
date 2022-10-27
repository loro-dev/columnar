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
