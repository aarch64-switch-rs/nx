//! Hardware-accelerated HMAC support (SHA-256 HMAC)

pub mod sha256;

/// Represents the IPAD value
pub const IPAD_VAL: u32 = 0x36363636;

/// Represents the OPAD value
pub const OPAD_VAL: u32 = 0x5C5C5C5C;

/// Representa the IPAD value xor'd with the OPAL value
pub const IPAD_XOR_OPAD_VAL: u32 = IPAD_VAL ^ OPAD_VAL;
