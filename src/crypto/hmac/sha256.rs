//! Hardware-accelerated SHA-256 HMAC support

use crate::crypto::{rc, sha256};
use crate::result::*;
use core::ptr;
use core::mem;

/// Represent the context used for SHA-256 HMAC operations
pub struct Context {
    sha_ctx: sha256::Context,
    key: [u32; sha256::BLOCK_SIZE_32],
    mac: [u32; sha256::HASH_SIZE_32],
    finalized: bool
}

impl Context {
    /// Creates a new [`Context`]
    /// 
    /// # Arguments
    /// 
    /// * `key`: The key to use
    pub fn new(key: &[u8]) -> Result<Self> {
        let mut ctx = Self {
            sha_ctx: sha256::Context::new(),
            key: [0; sha256::BLOCK_SIZE_32],
            mac: [0; sha256::HASH_SIZE_32],
            finalized: false
        };

        if key.len() <= sha256::BLOCK_SIZE {
            unsafe {
                ptr::copy(key.as_ptr(), ctx.key.as_mut_ptr() as *mut u8, key.len());
            }
        }
        else {
            ctx.sha_ctx.update(key);
            ctx.sha_ctx.get_hash(&mut ctx.key)?;
        }

        for i in 0..ctx.key.len() {
            ctx.key[i] ^= super::IPAD_VAL;
        }

        ctx.sha_ctx.reset();
        ctx.sha_ctx.update(&ctx.key);

        Ok(ctx)
    }

    /// Updates the [`Context`] with the given data
    /// 
    /// # Arguments
    /// 
    /// * `data`: The data to update with
    pub fn update<T>(&mut self, data: &[T]) {
        self.sha_ctx.update(data);
    }

    /// Gets the output MAC
    /// 
    /// The output hash array must have size [`HASH_SIZE`][`sha256::HASH_SIZE`] in bytes or this will fail with [`ResultInvalidSize`][`rc::ResultInvalidSize`]
    /// 
    /// # Arguments
    /// 
    /// * `out_mac`: Output array to fill into
    pub fn get_mac<T>(&mut self, out_mac: &mut [T]) -> Result<()> {
        result_return_unless!(out_mac.len() * mem::size_of::<T>() == sha256::HASH_SIZE, rc::ResultInvalidSize);

        if !self.finalized {
            self.sha_ctx.get_hash(&mut self.mac)?;

            for i in 0..self.key.len() {
                self.key[i] ^= super::IPAD_XOR_OPAD_VAL;
            }

            self.sha_ctx.reset();
            self.sha_ctx.update(&self.key);
            self.sha_ctx.update(&self.mac);
            self.sha_ctx.get_hash(&mut self.mac)?;

            self.finalized = true;
        }

        unsafe {
            ptr::copy(self.mac.as_ptr() as *const u8, out_mac.as_mut_ptr() as *mut u8, sha256::HASH_SIZE);
        }

        Ok(())
    }
}

/// Wrapper for directly calculating the MAC of given data
/// 
/// The output hash array must have size [`HASH_SIZE`][`sha256::HASH_SIZE`] in bytes or this will fail with [`ResultInvalidSize`][`rc::ResultInvalidSize`]
/// 
/// This essentially creates a [`Context`], updates it with the given data and produces its MAC
/// 
/// # Arguments
/// 
/// * `key`: Input key
/// * `data`: Input data
/// * `out_,ac`: Output array to fill into
#[inline]
pub fn calculate_mac<T>(key: &[u8], data: &[u8], out_mac: &mut [T]) -> Result<()> {
    let mut ctx = Context::new(key)?;
    ctx.update(data);
    ctx.get_mac(out_mac)
}