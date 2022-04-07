use crate::result::*;
use core::mem as cmem;

pub trait RandomGenerator {
    fn random_bytes(&mut self, buf: *mut u8, size: usize) -> Result<()>;

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

pub struct SplCsrngGenerator {
    csrng: mem::Shared<spl::RandomInterface>
}

impl SplCsrngGenerator {
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