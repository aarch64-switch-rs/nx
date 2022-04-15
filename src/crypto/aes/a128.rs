use core::mem;
use core::ptr;

pub const KEY_SIZE: usize = 0x10;
pub const KEY_SIZE_32: usize = KEY_SIZE / mem::size_of::<u32>();
pub const ROUND_COUNT: usize = 10;

pub struct Context {
    round_keys: [[u8; ROUND_COUNT + 1]; super::BLOCK_SIZE]
}

impl Context {
    pub fn new(key: &[u8], is_encryptor: bool) -> Self {
        unsafe {
            ptr::copy(src, dst, count)
        }
    }
}