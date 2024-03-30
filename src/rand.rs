//! Pseudo-RNG support

use crate::result::*;
use core::mem as cmem;

/// Represents a psudo-RNG
pub trait RandomGenerator {
    /// Fills the given memory region with random bytes
    /// 
    /// # Arguments
    /// 
    /// * `buf`: Memory region address
    /// * `size`: Memory region size
    fn random_bytes(&mut self, buf: *mut u8, size: usize) -> Result<()>;

    /// Generates a value filled with random contents
    /// 
    /// This is, of course, meant to be used with types where filling them with random data will be a valid value
    fn random<T: Copy + Default>(&mut self) -> Result<T> {
        let mut t: T = Default::default();
        self.random_bytes(&mut t as *mut _ as *mut u8, cmem::size_of::<T>())?;
        Ok(t)
    }
}

use crate::ipc::sf;
use crate::service;
use crate::service::spl;
use crate::service::spl::IRandomInterface;
use crate::mem;

/// Represents a pseudo-RNG using [`spl`]'s [`RandomInterface`][`spl::RandomInterface`] interface
pub struct SplCsrngGenerator {
    csrng: mem::Shared<dyn IRandomInterface>
}

impl SplCsrngGenerator {
    /// Creates a new [`SplCsrngGenerator`]
    pub fn new() -> Result<Self> {
        let csrng = service::new_service_object::<spl::RandomInterface>()?;
        Ok(Self { csrng })
    }
}

impl RandomGenerator for SplCsrngGenerator {
    fn random_bytes(&mut self, buf: *mut u8, size: usize) -> Result<()> {
        self.csrng.get().generate_random_bytes(sf::Buffer::from_mut_ptr(buf, size))
    }
}