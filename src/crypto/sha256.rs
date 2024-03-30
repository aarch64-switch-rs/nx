//! Hardware-accelerated SHA-256 support

use crate::crypto::rc;
use crate::result::*;
use core::mem;
use core::ptr;
use core::arch::asm;
use core::arch::aarch64;

/// Represents a block size in bytes
pub const BLOCK_SIZE: usize = 0x40;

/// Represent a block size un 4-byte units
pub const BLOCK_SIZE_32: usize = BLOCK_SIZE / mem::size_of::<u32>();

/// Represents a hash size in bytes
pub const HASH_SIZE: usize = 0x20;

/// Represents a hash size in 4-byte units
pub const HASH_SIZE_32: usize = HASH_SIZE / mem::size_of::<u32>();

/// Represents the 5context used for SHA-256 operations
pub struct Context {
    intermediate_hash: [u32; HASH_SIZE_32],
    buf: [u8; BLOCK_SIZE],
    bits_consumed: usize,
    buffered_size: usize,
    finalized: bool
}

const ROUND_CONSTANTS: [u32; 0x40] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2
];

const H_0: [u32; HASH_SIZE_32] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
];

impl Context {
    /// Creates a new [`Context`]
    pub fn new() -> Self {
        Self {
            intermediate_hash: H_0,
            buf: [0; BLOCK_SIZE],
            bits_consumed: 0,
            buffered_size: 0,
            finalized: false
        }
    }

    /// Resets this [`Context`]
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    unsafe fn process_blocks(&mut self, data: *const u8, block_count: usize) {
        let mut prev_hash0 = aarch64::vld1q_u32(self.intermediate_hash.as_ptr());
        let mut prev_hash1 = aarch64::vld1q_u32(self.intermediate_hash.as_ptr().offset(4));
        let mut cur_hash0 = aarch64::vdupq_n_u32(0);
        let mut cur_hash1 = aarch64::vdupq_n_u32(0);

        for _ in 0..block_count {
            let mut round_constant0 = aarch64::vdupq_n_u32(0);
            let mut round_constant1 = aarch64::vdupq_n_u32(0);
            let mut data0 = aarch64::vdupq_n_u32(0);
            let mut data1 = aarch64::vdupq_n_u32(0);
            let mut data2 = aarch64::vdupq_n_u32(0);
            let mut data3 = aarch64::vdupq_n_u32(0);
            let mut tmp0 = aarch64::vdupq_n_u32(0);
            let mut tmp1 = aarch64::vdupq_n_u32(0);
            let mut tmp2 = aarch64::vdupq_n_u32(0);
            let mut tmp3 = aarch64::vdupq_n_u32(0);
            let mut tmp_hash = aarch64::vdupq_n_u32(0);

            asm!(
                "ldp {data0:q}, {data1:q}, [{data}], #0x20",
                "ldp {data2:q}, {data3:q}, [{data}], #0x20",
                "add {cur_hash0:v}.4s, {cur_hash0:v}.4s, {prev_hash0:v}.4s",
                "ldp {round_constant0:q}, {round_constant1:q}, [{round_constants}, 0x00]",
                "add {cur_hash1:v}.4s, {cur_hash1:v}.4s, {prev_hash1:v}.4s",
                "rev32 {data0:v}.16b, {data0:v}.16b",
                "rev32 {data1:v}.16b, {data1:v}.16b",
                "rev32 {data2:v}.16b, {data2:v}.16b",
                "rev32 {data3:v}.16b, {data3:v}.16b",
                "add {tmp0:v}.4s, {data0:v}.4s, {round_constant0:v}.4s",
                "add {tmp1:v}.4s, {data1:v}.4s, {round_constant1:v}.4s",
                "ldp {round_constant0:q}, {round_constant1:q}, [{round_constants}, 0x20]",
                "sha256su0 {data0:v}.4s, {data1:v}.4s",
                "mov {prev_hash0:v}.16b, {cur_hash0:v}.16b",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp0:v}.4s",
                "mov {prev_hash1:v}.16b, {cur_hash1:v}.16b",
                "sha256h2 {cur_hash1:q}, {prev_hash0:q}, {tmp0:v}.4s",
                "sha256su0 {data1:v}.4s, {data2:v}.4s",
                "sha256su1 {data0:v}.4s, {data2:v}.4s, {data3:v}.4s",
                "add {tmp2:v}.4s, {data2:v}.4s, {round_constant0:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp1:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp1:v}.4s",
                "sha256su0 {data2:v}.4s, {data3:v}.4s",
                "sha256su1 {data1:v}.4s, {data3:v}.4s, {data0:v}.4s",
                "add {tmp3:v}.4s, {data3:v}.4s, {round_constant1:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "ldp {round_constant0:q}, {round_constant1:q}, [{round_constants}, 0x40]",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp2:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp2:v}.4s",
                "sha256su0 {data3:v}.4s, {data0:v}.4s",
                "sha256su1 {data2:v}.4s, {data0:v}.4s, {data1:v}.4s",
                "add {tmp0:v}.4s, {data0:v}.4s, {round_constant0:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp3:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp3:v}.4s",
                "sha256su0 {data0:v}.4s, {data1:v}.4s",
                "sha256su1 {data3:v}.4s, {data1:v}.4s, {data2:v}.4s",
                "add {tmp1:v}.4s, {data1:v}.4s, {round_constant1:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "ldp {round_constant0:q}, {round_constant1:q}, [{round_constants}, 0x60]",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp0:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp0:v}.4s",
                "sha256su0 {data1:v}.4s, {data2:v}.4s",
                "sha256su1 {data0:v}.4s, {data2:v}.4s, {data3:v}.4s",
                "add {tmp2:v}.4s, {data2:v}.4s, {round_constant0:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp1:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp1:v}.4s",
                "sha256su0 {data2:v}.4s, {data3:v}.4s",
                "sha256su1 {data1:v}.4s, {data3:v}.4s, {data0:v}.4s",
                "add {tmp3:v}.4s, {data3:v}.4s, {round_constant1:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "ldp {round_constant0:q}, {round_constant1:q}, [{round_constants}, 0x80]",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp2:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp2:v}.4s",
                "sha256su0 {data3:v}.4s, {data0:v}.4s",
                "sha256su1 {data2:v}.4s, {data0:v}.4s, {data1:v}.4s",
                "add {tmp0:v}.4s, {data0:v}.4s, {round_constant0:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp3:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp3:v}.4s",
                "sha256su0 {data0:v}.4s, {data1:v}.4s",
                "sha256su1 {data3:v}.4s, {data1:v}.4s, {data2:v}.4s",
                "add {tmp1:v}.4s, {data1:v}.4s, {round_constant1:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "ldp {round_constant0:q}, {round_constant1:q}, [{round_constants}, 0xA0]",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp0:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp0:v}.4s",
                "sha256su0 {data1:v}.4s, {data2:v}.4s",
                "sha256su1 {data0:v}.4s, {data2:v}.4s, {data3:v}.4s",
                "add {tmp2:v}.4s, {data2:v}.4s, {round_constant0:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp1:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp1:v}.4s",
                "sha256su0 {data2:v}.4s, {data3:v}.4s",
                "sha256su1 {data1:v}.4s, {data3:v}.4s, {data0:v}.4s",
                "add {tmp3:v}.4s, {data3:v}.4s, {round_constant1:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "ldp {round_constant0:q}, {round_constant1:q}, [{round_constants}, 0xC0]",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp2:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp2:v}.4s",
                "sha256su0 {data3:v}.4s, {data0:v}.4s",
                "sha256su1 {data2:v}.4s, {data0:v}.4s, {data1:v}.4s",
                "add {tmp0:v}.4s, {data0:v}.4s, {round_constant0:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp3:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp3:v}.4s",
                "sha256su1 {data3:v}.4s, {data1:v}.4s, {data2:v}.4s",
                "add {tmp1:v}.4s, {data1:v}.4s, {round_constant1:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "ldp {round_constant0:q}, {round_constant1:q}, [{round_constants}, 0xE0]",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp0:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp0:v}.4s",
                "add {tmp2:v}.4s, {data2:v}.4s, {round_constant0:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp1:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp1:v}.4s",
                "add {tmp3:v}.4s, {data3:v}.4s, {round_constant1:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp2:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp2:v}.4s",
                "mov {tmp_hash:v}.16b, {cur_hash0:v}.16b",
                "sha256h {cur_hash0:q}, {cur_hash1:q}, {tmp3:v}.4s",
                "sha256h2 {cur_hash1:q}, {tmp_hash:q}, {tmp3:v}.4s",
                data0 = inout(vreg) data0,
                data1 = inout(vreg) data1,
                data2 = inout(vreg) data2,
                data3 = inout(vreg) data3,
                data = in(reg) data,
                tmp0 = inout(vreg) tmp0,
                tmp1 = inout(vreg) tmp1,
                tmp2 = inout(vreg) tmp2,
                tmp3 = inout(vreg) tmp3,
                round_constant0 = inout(vreg) round_constant0,
                round_constant1 = inout(vreg) round_constant1,
                cur_hash0 = inout(vreg) cur_hash0,
                cur_hash1 = inout(vreg) cur_hash1,
                prev_hash0 = inout(vreg) prev_hash0,
                prev_hash1 = inout(vreg) prev_hash1,
                tmp_hash = inout(vreg) tmp_hash,
                round_constants = in(reg) ROUND_CONSTANTS.as_ptr()
            );

            // Silence warnings of the last assigned value not being read
            let _ = data0;
            let _ = data1;
            let _ = data2;
            let _ = data3;
            let _ = tmp0;
            let _ = tmp1;
            let _ = tmp2;
            let _ = tmp3;
            let _ = round_constant0;
            let _ = round_constant1;
            let _ = tmp_hash;
        }

        cur_hash0 = aarch64::vaddq_u32(prev_hash0, cur_hash0);
        cur_hash1 = aarch64::vaddq_u32(prev_hash1, cur_hash1);
        aarch64::vst1q_u32(self.intermediate_hash.as_mut_ptr(), cur_hash0);
        aarch64::vst1q_u32(self.intermediate_hash.as_mut_ptr().offset(4), cur_hash1);
    }

    /// Updates the [`Context`] with the given data
    /// 
    /// # Arguments
    /// 
    /// * `data`: The data to update with
    pub fn update<T>(&mut self, data: &[T]) {
        let data_size = data.len() * mem::size_of::<T>();
        let data_start = data.as_ptr() as *const u8;
        self.bits_consumed += (((self.buffered_size + data_size) / BLOCK_SIZE) * BLOCK_SIZE) * 8;
        let mut data_offset: usize = 0;
        let mut cur_size = data_size;

        if self.buffered_size > 0 {
            let needed = BLOCK_SIZE - self.buffered_size;

            let copyable = needed.min(cur_size);
            unsafe {
                ptr::copy(data_start.offset(data_offset as isize), self.buf.as_mut_ptr().offset(self.buffered_size as isize), copyable);
            }
            data_offset += copyable;
            cur_size -= copyable;

            if self.buffered_size == BLOCK_SIZE {
                unsafe {
                    self.process_blocks(self.buf.as_ptr(), 1);
                }
                self.buffered_size = 0;
            }
        }

        if cur_size >= BLOCK_SIZE {
            let block_count = cur_size / BLOCK_SIZE;
            unsafe {
                self.process_blocks(data_start.offset(data_offset as isize), block_count);
            }
            let blocks_size = BLOCK_SIZE * block_count;
            data_offset += blocks_size;
            cur_size -= blocks_size;
        }

        if cur_size > 0 {
            unsafe {
                ptr::copy(data_start.offset(data_offset as isize), self.buf.as_mut_ptr(), cur_size);
            }
            self.buffered_size = cur_size;
        }
    }

    /// Gets the produced hash (produces it first if not done yet)
    /// 
    /// The output hash array must have size [`HASH_SIZE`] in bytes or this will fail with [`ResultInvalidSize`][`rc::ResultInvalidSize`]
    /// 
    /// # Arguments
    /// 
    /// * `out_hash`: The output array to fill with the hash 
    pub fn get_hash<T>(&mut self, out_hash: &mut [T]) -> Result<()> {
        result_return_unless!(out_hash.len() * mem::size_of::<T>() == HASH_SIZE, rc::ResultInvalidSize);

        if !self.finalized {
            // Process last block, if necessary
            self.bits_consumed += 8 * self.buffered_size;
            self.buf[self.buffered_size] = 0x80;
            self.buffered_size += 1;

            let last_block_max_size = BLOCK_SIZE - mem::size_of::<u64>();
            unsafe {
                if self.buffered_size <= last_block_max_size {
                    ptr::write_bytes(self.buf.as_mut_ptr().offset(self.buffered_size as isize), 0, last_block_max_size - self.buffered_size);
                }
                else {
                    ptr::write_bytes(self.buf.as_mut_ptr().offset(self.buffered_size as isize), 0, BLOCK_SIZE - self.buffered_size);
                    self.process_blocks(self.buf.as_ptr(), 1);

                    ptr::write_bytes(self.buf.as_mut_ptr(), 0, last_block_max_size);
                }

                let be_bits_consumed = self.bits_consumed.swap_bytes();
                *(self.buf.as_mut_ptr().offset(last_block_max_size as isize) as *mut usize) = be_bits_consumed;
                self.process_blocks(self.buf.as_ptr(), 1);
            }
            self.finalized = true;
        }

        unsafe {
            let out_hash_buf_32 = out_hash.as_mut_ptr() as *mut u32;
            for i in 0..HASH_SIZE_32 {
                *out_hash_buf_32.offset(i as isize) = self.intermediate_hash[i].swap_bytes();
            }
        }

        Ok(())
    }
}

/// Wrapper for directly calculating the hash of given data
/// 
/// The output hash array must have size [`HASH_SIZE`] in bytes or this will fail with [`ResultInvalidSize`][`rc::ResultInvalidSize`]
/// 
/// This essentially creates a [`Context`], updates it with the given data and produces its hash
/// 
/// # Arguments
/// 
/// * `data`: Input data
/// * `out_hash`: Output array to fill into
#[inline]
pub fn calculate_hash<T, U>(data: &[T], out_hash: &mut [U]) -> Result<()> {
    let mut ctx = Context::new();
    ctx.update(data);
    ctx.get_hash(out_hash)
}