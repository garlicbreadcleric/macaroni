#[inline]
pub const fn is_continuation_byte(byte: u8) -> bool {
  (byte as i8) < -64
}
