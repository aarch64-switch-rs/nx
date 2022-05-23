use crate::crypto::rc;
use crate::result::*;
use core::mem;
use core::ptr;
use core::arch::asm;
use core::arch::aarch64;

pub const KEY_SIZE: usize = 0x10;
pub const KEY_SIZE_32: usize = KEY_SIZE / mem::size_of::<u32>();
pub const ROUND_COUNT: usize = 10;

const SUB_BYTES_TABLE: [u8; 0x100] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16
];

const RCON_TABLE: [u8; 0x10] = [
    0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a, 0x2f
];

#[inline(always)]
const fn sub_bytes(tmp: u32) -> u32 {
    ((SUB_BYTES_TABLE[((tmp >> 0x00) & 0xFF) as usize] as u32) << 0x00) |
    ((SUB_BYTES_TABLE[((tmp >> 0x08) & 0xFF) as usize] as u32) << 0x08) |
    ((SUB_BYTES_TABLE[((tmp >> 0x10) & 0xFF) as usize] as u32) << 0x10) |
    ((SUB_BYTES_TABLE[((tmp >> 0x18) & 0xFF) as usize] as u32) << 0x18)
}

#[inline(always)]
const fn rotate_bytes(tmp: u32) -> u32 {
    (((tmp >> 0x00) & 0xFF) << 0x18) |
    (((tmp >> 0x08) & 0xFF) << 0x00) |
    (((tmp >> 0x10) & 0xFF) << 0x08) |
    (((tmp >> 0x18) & 0xFF) << 0x10)
}

pub struct Context {
    pub round_keys: [[u8; ROUND_COUNT + 1]; super::BLOCK_SIZE]
}

impl Context {
    pub fn new(key: &[u8], is_encryptor: bool) -> Result<Self> {
        result_return_unless!(key.len() == KEY_SIZE, rc::ResultInvalidSize);
        // TODO: ensure key.len() == KEY_SIZE

        let mut ctx = Self {
            round_keys: [[0; ROUND_COUNT + 1]; super::BLOCK_SIZE]
        };

        let round_keys_32 = ctx.round_keys.as_mut_ptr() as *mut u32;

        unsafe {
            ptr::copy(key.as_ptr(), round_keys_32 as *mut u8, KEY_SIZE);
        }

        let mut tmp = unsafe {
            *round_keys_32.add(KEY_SIZE_32 - 1)
        };


        let round_keys_size_32 = (super::BLOCK_SIZE * (ROUND_COUNT + 1)) / mem::size_of::<u32>();
        for i in KEY_SIZE_32..round_keys_size_32 {
            if (i % KEY_SIZE_32) == 0 {
                tmp = rotate_bytes(sub_bytes(tmp)) ^ (RCON_TABLE[(i / KEY_SIZE_32) - 1] as u32);
            }

            tmp ^= unsafe {
                *round_keys_32.add(i - KEY_SIZE_32)
            };
            
            unsafe {
                *round_keys_32.add(i) = tmp;
            }
        }

        if !is_encryptor {
            for i in 1..ROUND_COUNT {
                unsafe {
                    let mut tmp_key = aarch64::vld1q_u8(ctx.round_keys[i].as_ptr());
                    tmp_key = aarch64::vaesimcq_u8(tmp_key);
                    aarch64::vst1q_u8(ctx.round_keys[i].as_mut_ptr(), tmp_key);
                }
            }
        }

        Ok(ctx)
    }

    pub fn encrypt_block(&self, src: &[u8], dst: &mut [u8]) {
        unsafe {
            let mut tmp = aarch64::vld1q_u8(src.as_ptr());
            let mut tmp2 = aarch64::vdupq_n_u8(0);

            asm!(
                "ldr {tmp2:q}, [{round_key_1}]",
                "aese {tmp:v}.16b, {tmp2:v}.16b",
                "aesmc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_2}]",
                "aese {tmp:v}.16b, {tmp2:v}.16b",
                "aesmc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_3}]",
                "aese {tmp:v}.16b, {tmp2:v}.16b",
                "aesmc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_4}]",
                "aese {tmp:v}.16b, {tmp2:v}.16b",
                "aesmc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_5}]",
                "aese {tmp:v}.16b, {tmp2:v}.16b",
                "aesmc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_6}]",
                "aese {tmp:v}.16b, {tmp2:v}.16b",
                "aesmc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_7}]",
                "aese {tmp:v}.16b, {tmp2:v}.16b",
                "aesmc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_8}]",
                "aese {tmp:v}.16b, {tmp2:v}.16b",
                "aesmc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_9}]",
                "aese {tmp:v}.16b, {tmp2:v}.16b",
                "aesmc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_second_last}]",
                "aese {tmp:v}.16b, {tmp2:v}.16b",

                "ldr {tmp2:q}, [{round_key_last}]",
                "eor {tmp:v}.16b, {tmp:v}.16b, {tmp2:v}.16b",

                tmp = inout(vreg) tmp,
                tmp2 = inout(vreg) tmp2,
                round_key_1 = in(reg) self.round_keys[0].as_ptr(),
                round_key_2 = in(reg) self.round_keys[1].as_ptr(),
                round_key_3 = in(reg) self.round_keys[2].as_ptr(),
                round_key_4 = in(reg) self.round_keys[3].as_ptr(),
                round_key_5 = in(reg) self.round_keys[4].as_ptr(),
                round_key_6 = in(reg) self.round_keys[5].as_ptr(),
                round_key_7 = in(reg) self.round_keys[6].as_ptr(),
                round_key_8 = in(reg) self.round_keys[7].as_ptr(),
                round_key_9 = in(reg) self.round_keys[8].as_ptr(),
                round_key_second_last = in(reg) self.round_keys[9].as_ptr(),
                round_key_last = in(reg) self.round_keys[10].as_ptr()
            );

            // Silence warnings of the last assigned value not being read
            let _ = tmp2;

            aarch64::vst1q_u8(dst.as_mut_ptr(), tmp);
        }
    }

    pub fn decrypt_block(&self, src: &[u8], dst: &mut [u8]) {
        unsafe {
            let mut tmp = aarch64::vld1q_u8(src.as_ptr());
            let mut tmp2 = aarch64::vdupq_n_u8(0);

            asm!(
                "ldr {tmp2:q}, [{round_key_1}]",
                "aesd {tmp:v}.16b, {tmp2:v}.16b",
                "aesimc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_2}]",
                "aesd {tmp:v}.16b, {tmp2:v}.16b",
                "aesimc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_3}]",
                "aesd {tmp:v}.16b, {tmp2:v}.16b",
                "aesimc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_4}]",
                "aesd {tmp:v}.16b, {tmp2:v}.16b",
                "aesimc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_5}]",
                "aesd {tmp:v}.16b, {tmp2:v}.16b",
                "aesimc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_6}]",
                "aesd {tmp:v}.16b, {tmp2:v}.16b",
                "aesimc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_7}]",
                "aesd {tmp:v}.16b, {tmp2:v}.16b",
                "aesimc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_8}]",
                "aesd {tmp:v}.16b, {tmp2:v}.16b",
                "aesimc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_9}]",
                "aesd {tmp:v}.16b, {tmp2:v}.16b",
                "aesimc {tmp:v}.16b, {tmp:v}.16b",

                "ldr {tmp2:q}, [{round_key_second_last}]",
                "aesd {tmp:v}.16b, {tmp2:v}.16b",

                "ldr {tmp2:q}, [{round_key_last}]",
                "eor {tmp:v}.16b, {tmp:v}.16b, {tmp2:v}.16b",

                tmp = inout(vreg) tmp,
                tmp2 = inout(vreg) tmp2,
                round_key_1 = in(reg) self.round_keys[10].as_ptr(),
                round_key_2 = in(reg) self.round_keys[9].as_ptr(),
                round_key_3 = in(reg) self.round_keys[8].as_ptr(),
                round_key_4 = in(reg) self.round_keys[7].as_ptr(),
                round_key_5 = in(reg) self.round_keys[6].as_ptr(),
                round_key_6 = in(reg) self.round_keys[5].as_ptr(),
                round_key_7 = in(reg) self.round_keys[4].as_ptr(),
                round_key_8 = in(reg) self.round_keys[3].as_ptr(),
                round_key_9 = in(reg) self.round_keys[2].as_ptr(),
                round_key_second_last = in(reg) self.round_keys[1].as_ptr(),
                round_key_last = in(reg) self.round_keys[0].as_ptr()
            );

            // Silence warnings of the last assigned value not being read
            let _ = tmp2;

            aarch64::vst1q_u8(dst.as_mut_ptr(), tmp);
        }
    }
}