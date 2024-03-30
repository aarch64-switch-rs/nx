//! hardware-accelerated AES support (normal and CTR)

pub mod a128;

pub mod ctr;

/// Represents the block size for common AES operations
pub const BLOCK_SIZE: usize = 0x10;