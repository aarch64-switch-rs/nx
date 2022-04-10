pub mod sha256;

pub const IPAD_VAL: u32 = 0x36363636;
pub const OPAD_VAL: u32 = 0x5C5C5C5C;
pub const IPAD_XOR_OPAD_VAL: u32 = IPAD_VAL ^ OPAD_VAL;