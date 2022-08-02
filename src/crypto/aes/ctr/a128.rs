//! Hardware-accelerated 128-bit AES-CTR support

use crate::crypto::aes;
use crate::crypto::rc;
use crate::result::*;
use core::ptr;
use core::arch::aarch64;
use core::arch::asm;

unsafe fn increment_ctr(ctr: aarch64::uint8x16_t) -> aarch64::uint8x16_t {
    let mut inc = aarch64::vdupq_n_u8(0);
    let mut high = 0u64;
    let mut low = 0u64;

    asm!(
        "mov {high}, {ctr:v}.d[0]",
        "mov {low}, {ctr:v}.d[1]",
        "rev {high}, {high}",
        "rev {low}, {low}",
        "adds {low}, {low}, 1",
        "cinc {high}, {high}, cs",
        "rev {high}, {high}",
        "rev {low}, {low}",
        "mov {inc:v}.d[0], {high}",
        "mov {inc:v}.d[1], {low}",

        inc = inout(vreg) inc,
        high = inout(reg) high,
        low = inout(reg) low,
        ctr = in(vreg) ctr
    );

    // Silence warnings of the last assigned value not being read
    let _ = high;
    let _ = low;

    inc
}

/// Represents the context used for 128-bit AES-CTR operations
pub struct Context {
    aes_ctx: aes::a128::Context,
    ctr: [u8; aes::BLOCK_SIZE],
    enc_ctr_buffer: [u8; aes::BLOCK_SIZE],
    buffer_offset: usize
}

impl Context {
    /// Creates a new [`Context`]
    /// 
    /// The key must have size [`KEY_SIZE`][`aes::a128::KEY_SIZE`] in bytes and the ctr must have size [`BLOCK_SIZE`][`aes::BLOCK_SIZE`] in bytes or this will fail with [`ResultInvalidSize`][`rc::ResultInvalidSize`]
    /// 
    /// # Arguments
    /// 
    /// * `key`: The 128-bit AES key
    /// * `ctr`: The ctr
    pub fn new(key: &[u8], ctr: &[u8]) -> Result<Self> {
        let mut ctx = Self {
            aes_ctx: aes::a128::Context::new(key, true)?,
            ctr: [0; aes::BLOCK_SIZE],
            enc_ctr_buffer: [0; aes::BLOCK_SIZE],
            buffer_offset: 0
        };

        ctx.reset_ctr(ctr)?;

        Ok(ctx)
    }

    /// Resets the [`Context`] with the provided ctr
    /// 
    /// The ctr must have size [`BLOCK_SIZE`][`aes::BLOCK_SIZE`] in bytes or this will fail with [`ResultInvalidSize`][`rc::ResultInvalidSize`]
    /// 
    /// # Argument
    /// 
    /// * `ctr`: The ctr to reset with
    pub fn reset_ctr(&mut self, ctr: &[u8]) -> Result<()> {
        result_return_unless!(ctr.len() == aes::BLOCK_SIZE, rc::ResultInvalidSize);

        unsafe {
            ptr::copy(ctr.as_ptr(), self.ctr.as_mut_ptr(), ctr.len());
        }
        self.enc_ctr_buffer = [0; aes::BLOCK_SIZE];
        self.buffer_offset = 0;
        Ok(())
    }

    unsafe fn crypt_blocks(&mut self, src: *const u8, dst: *mut u8, block_count: usize) {
        let round_key_0 = aarch64::vld1q_u8(self.aes_ctx.round_keys[0].as_ptr());
        let round_key_1 = aarch64::vld1q_u8(self.aes_ctx.round_keys[1].as_ptr());
        let round_key_2 = aarch64::vld1q_u8(self.aes_ctx.round_keys[2].as_ptr());
        let round_key_3 = aarch64::vld1q_u8(self.aes_ctx.round_keys[3].as_ptr());
        let round_key_4 = aarch64::vld1q_u8(self.aes_ctx.round_keys[4].as_ptr());
        let round_key_5 = aarch64::vld1q_u8(self.aes_ctx.round_keys[5].as_ptr());
        let round_key_6 = aarch64::vld1q_u8(self.aes_ctx.round_keys[6].as_ptr());
        let round_key_7 = aarch64::vld1q_u8(self.aes_ctx.round_keys[7].as_ptr());
        let round_key_8 = aarch64::vld1q_u8(self.aes_ctx.round_keys[8].as_ptr());
        let round_key_9 = aarch64::vld1q_u8(self.aes_ctx.round_keys[9].as_ptr());
        let round_key_10 = aarch64::vld1q_u8(self.aes_ctx.round_keys[10].as_ptr());

        let mut ctr0 = aarch64::vld1q_u8(self.ctr.as_ptr());

        let mut high = 0u64;
        let mut low = 0u64;

        let mut cur_src = src;
        let mut cur_dst = dst;
        let mut cur_count = block_count; 
        if cur_count >= 3 {
            let mut ctr1 = increment_ctr(ctr0);
            let mut ctr2 = increment_ctr(ctr1);
            let mut high_tmp = 0u64;
            let mut low_tmp = 0u64;

            while cur_count >= 3 {
                let block0 = aarch64::vld1q_u8(cur_src);
                cur_src = cur_src.add(aes::BLOCK_SIZE);
                let block1 = aarch64::vld1q_u8(cur_src);
                cur_src = cur_src.add(aes::BLOCK_SIZE);
                let block2 = aarch64::vld1q_u8(cur_src);
                cur_src = cur_src.add(aes::BLOCK_SIZE);

                let mut tmp0 = ctr0;
                let mut tmp1 = ctr1;
                let mut tmp2 = ctr2;

                asm!(
                    "aese {tmp0:v}.16b, {round_key_0:v}.16b",
                    "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                    "mov {high}, {ctr2:v}.d[0]",

                    "aese {tmp1:v}.16b, {round_key_0:v}.16b",
                    "aesmc {tmp1:v}.16b, {tmp1:v}.16b",

                    "mov {low}, {ctr2:v}.d[1]",

                    "aese {tmp2:v}.16b, {round_key_0:v}.16b",
                    "aesmc {tmp2:v}.16b, {tmp2:v}.16b",

                    "rev {high}, {high}",

                    "aese {tmp0:v}.16b, {round_key_1:v}.16b",
                    "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                    "rev {low}, {low}",

                    "aese {tmp1:v}.16b, {round_key_1:v}.16b",
                    "aesmc {tmp1:v}.16b, {tmp1:v}.16b",

                    "adds {low}, {low}, 1",

                    "aese {tmp2:v}.16b, {round_key_1:v}.16b",
                    "aesmc {tmp2:v}.16b, {tmp2:v}.16b",

                    "cinc {high}, {high}, cs",

                    "aese {tmp0:v}.16b, {round_key_2:v}.16b",
                    "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                    "rev {high_tmp}, {high}",
                    
                    "aese {tmp1:v}.16b, {round_key_2:v}.16b",
                    "aesmc {tmp1:v}.16b, {tmp1:v}.16b",

                    "rev {low_tmp}, {low}",

                    "aese {tmp2:v}.16b, {round_key_2:v}.16b",
                    "aesmc {tmp2:v}.16b, {tmp2:v}.16b",

                    "mov {ctr0:v}.d[0], {high_tmp}",

                    "aese {tmp0:v}.16b, {round_key_3:v}.16b",
                    "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                    "mov {ctr0:v}.d[1], {low_tmp}",

                    "aese {tmp1:v}.16b, {round_key_3:v}.16b",
                    "aesmc {tmp1:v}.16b, {tmp1:v}.16b",

                    "adds {low}, {low}, 1",

                    "aese {tmp2:v}.16b, {round_key_3:v}.16b",
                    "aesmc {tmp2:v}.16b, {tmp2:v}.16b",

                    "cinc {high}, {high}, cs",

                    "aese {tmp0:v}.16b, {round_key_4:v}.16b",
                    "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                    "rev {high_tmp}, {high}",

                    "aese {tmp1:v}.16b, {round_key_4:v}.16b",
                    "aesmc {tmp1:v}.16b, {tmp1:v}.16b",

                    "rev {low_tmp}, {low}",

                    "aese {tmp2:v}.16b, {round_key_4:v}.16b",
                    "aesmc {tmp2:v}.16b, {tmp2:v}.16b",

                    "mov {ctr1:v}.d[0], {high_tmp}",

                    "aese {tmp0:v}.16b, {round_key_5:v}.16b",
                    "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                    "mov {ctr1:v}.d[1], {low_tmp}",

                    "aese {tmp1:v}.16b, {round_key_5:v}.16b",
                    "aesmc {tmp1:v}.16b, {tmp1:v}.16b",

                    "adds {low}, {low}, 1",

                    "aese {tmp2:v}.16b, {round_key_5:v}.16b",
                    "aesmc {tmp2:v}.16b, {tmp2:v}.16b",

                    "cinc {high}, {high}, cs",

                    "aese {tmp0:v}.16b, {round_key_6:v}.16b",
                    "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                    "rev {high_tmp}, {high}",

                    "aese {tmp1:v}.16b, {round_key_6:v}.16b",
                    "aesmc {tmp1:v}.16b, {tmp1:v}.16b",

                    "rev {low_tmp}, {low}",

                    "aese {tmp2:v}.16b, {round_key_6:v}.16b",
                    "aesmc {tmp2:v}.16b, {tmp2:v}.16b",

                    "mov {ctr2:v}.d[0], {high_tmp}",

                    "aese {tmp0:v}.16b, {round_key_7:v}.16b",
                    "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                    "mov {ctr2:v}.d[1], {low_tmp}",

                    "aese {tmp1:v}.16b, {round_key_7:v}.16b",
                    "aesmc {tmp1:v}.16b, {tmp1:v}.16b",

                    "aese {tmp2:v}.16b, {round_key_7:v}.16b",
                    "aesmc {tmp2:v}.16b, {tmp2:v}.16b",

                    "aese {tmp0:v}.16b, {round_key_8:v}.16b",
                    "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                    "aese {tmp1:v}.16b, {round_key_8:v}.16b",
                    "aesmc {tmp1:v}.16b, {tmp1:v}.16b",

                    "aese {tmp2:v}.16b, {round_key_8:v}.16b",
                    "aesmc {tmp2:v}.16b, {tmp2:v}.16b",

                    "aese {tmp0:v}.16b, {round_key_9:v}.16b",
                    
                    "aese {tmp1:v}.16b, {round_key_9:v}.16b",
                    
                    "aese {tmp2:v}.16b, {round_key_9:v}.16b",

                    "eor {tmp0:v}.16b, {tmp0:v}.16b, {round_key_10:v}.16b",

                    "eor {tmp1:v}.16b, {tmp1:v}.16b, {round_key_10:v}.16b",

                    "eor {tmp2:v}.16b, {tmp2:v}.16b, {round_key_10:v}.16b",

                    round_key_0 = in(vreg) round_key_0,
                    round_key_1 = in(vreg) round_key_1,
                    round_key_2 = in(vreg) round_key_2,
                    round_key_3 = in(vreg) round_key_3,
                    round_key_4 = in(vreg) round_key_4,
                    round_key_5 = in(vreg) round_key_5,
                    round_key_6 = in(vreg) round_key_6,
                    round_key_7 = in(vreg) round_key_7,
                    round_key_8 = in(vreg) round_key_8,
                    round_key_9 = in(vreg) round_key_9,
                    round_key_10 = in(vreg) round_key_10,

                    tmp0 = inout(vreg) tmp0,
                    tmp1 = inout(vreg) tmp1,
                    tmp2 = inout(vreg) tmp2,

                    high = inout(reg) high,
                    low = inout(reg) low,
                    high_tmp = inout(reg) high_tmp,
                    low_tmp = inout(reg) low_tmp,

                    ctr0 = inout(vreg) ctr0,
                    ctr1 = inout(vreg) ctr1,
                    ctr2 = inout(vreg) ctr2
                );

                tmp0 = aarch64::veorq_u8(block0, tmp0);
                tmp1 = aarch64::veorq_u8(block1, tmp1);
                tmp2 = aarch64::veorq_u8(block2, tmp2);

                aarch64::vst1q_u8(cur_dst, tmp0);
                cur_dst = cur_dst.add(aes::BLOCK_SIZE);
                aarch64::vst1q_u8(cur_dst, tmp1);
                cur_dst = cur_dst.add(aes::BLOCK_SIZE);
                aarch64::vst1q_u8(cur_dst, tmp2);
                cur_dst = cur_dst.add(aes::BLOCK_SIZE);

                cur_count -= 3;
            }
        }

        while cur_count >= 1 {
            let block0 = aarch64::vld1q_u8(cur_src);
            cur_src = cur_src.add(aes::BLOCK_SIZE);

            let mut tmp0 = ctr0;

            asm!(
                "aese {tmp0:v}.16b, {round_key_0:v}.16b",
                "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                "mov {high}, {ctr0:v}.d[0]",

                "aese {tmp0:v}.16b, {round_key_1:v}.16b",
                "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                "mov {low}, {ctr0:v}.d[1]",

                "aese {tmp0:v}.16b, {round_key_2:v}.16b",
                "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                "rev {high}, {high}",

                "aese {tmp0:v}.16b, {round_key_3:v}.16b",
                "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                "rev {low}, {low}",

                "aese {tmp0:v}.16b, {round_key_4:v}.16b",
                "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                "adds {low}, {low}, 1",

                "aese {tmp0:v}.16b, {round_key_5:v}.16b",
                "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                "cinc {high}, {high}, cs",

                "aese {tmp0:v}.16b, {round_key_6:v}.16b",
                "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                "rev {high}, {high}",

                "aese {tmp0:v}.16b, {round_key_7:v}.16b",
                "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                "rev {low}, {low}",

                "aese {tmp0:v}.16b, {round_key_8:v}.16b",
                "aesmc {tmp0:v}.16b, {tmp0:v}.16b",

                "mov {ctr0:v}.d[0], {high}",

                "aese {tmp0:v}.16b, {round_key_9:v}.16b",

                "mov {ctr0:v}.d[1], {low}",
        
                "eor {tmp0:v}.16b, {tmp0:v}.16b, {round_key_10:v}.16b",

                round_key_0 = in(vreg) round_key_0,
                round_key_1 = in(vreg) round_key_1,
                round_key_2 = in(vreg) round_key_2,
                round_key_3 = in(vreg) round_key_3,
                round_key_4 = in(vreg) round_key_4,
                round_key_5 = in(vreg) round_key_5,
                round_key_6 = in(vreg) round_key_6,
                round_key_7 = in(vreg) round_key_7,
                round_key_8 = in(vreg) round_key_8,
                round_key_9 = in(vreg) round_key_9,
                round_key_10 = in(vreg) round_key_10,

                tmp0 = inout(vreg) tmp0,

                high = inout(reg) high,
                low = inout(reg) low,

                ctr0 = inout(vreg) ctr0
            );
            
            tmp0 = aarch64::veorq_u8(block0, tmp0);

            aarch64::vst1q_u8(cur_dst, tmp0);
            cur_dst = cur_dst.add(aes::BLOCK_SIZE);

            cur_count -= 1;
        }

        aarch64::vst1q_u8(self.ctr.as_mut_ptr(), ctr0);
    }

    /// Crypts the given data
    /// 
    /// Input and output data must have the same size, or this will fail with [`ResultInvalidSize`][`rc::ResultInvalidSize`]
    /// 
    /// # Arguments
    /// 
    /// * `src`: The input data
    /// * `dst`: The output data to fill into
    pub fn crypt(&mut self, src: &[u8], dst: &mut [u8]) -> Result<()> {
        result_return_unless!(src.len() == dst.len(), rc::ResultInvalidSize);

        let mut cur_src = src.as_ptr();
        let mut cur_dst = dst.as_mut_ptr();
        let mut cur_size = src.len();

        if self.buffer_offset > 0 {
            let needed = aes::BLOCK_SIZE - self.buffer_offset;
            let copyable = cur_size.min(needed);

            unsafe {
                for i in 0..copyable {
                    *cur_dst.add(i) = *cur_src.add(i) ^ self.enc_ctr_buffer[self.buffer_offset + i];
                }

                cur_dst = cur_dst.add(copyable);
                cur_src = cur_src.add(copyable);
                self.buffer_offset += copyable;
                cur_size -= copyable;

                if self.buffer_offset == aes::BLOCK_SIZE {
                    self.buffer_offset = 0;
                }
            }
        }

        unsafe {
            if cur_size >= aes::BLOCK_SIZE {
                let block_count = cur_size / aes::BLOCK_SIZE;
                self.crypt_blocks(cur_src, cur_dst, block_count);

                let blocks_size = block_count * aes::BLOCK_SIZE;
                cur_size -= blocks_size;
                cur_src = cur_src.add(blocks_size);
                cur_dst = cur_dst.add(blocks_size);
            }

            if cur_size > 0 {
                ptr::copy(cur_src, self.enc_ctr_buffer.as_mut_ptr(), cur_size);
                ptr::write_bytes(self.enc_ctr_buffer.as_mut_ptr().add(cur_size), 0, aes::BLOCK_SIZE - cur_size);

                let enc_ctr_buf_ptr = self.enc_ctr_buffer.as_ptr();
                self.crypt_blocks(enc_ctr_buf_ptr, enc_ctr_buf_ptr as *mut u8, 1);

                ptr::copy(enc_ctr_buf_ptr, cur_dst, cur_size);
                self.buffer_offset = cur_size;
            }
        }

        Ok(())
    }
}