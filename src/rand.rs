//! Pseudo-RNG support

use core::convert::Infallible;

use alloc::sync::Arc;
use rand::TryRng;
pub use rand::{CryptoRng, Rng, rand_core::Rng as RngCore};

/// Represents a pseudo-RNG
use crate::ipc::sf::Buffer;
use crate::result::*;
use crate::service;
pub use crate::service::spl::{IRandomClient, RandomService};
use crate::sync::Mutex;

impl TryRng for RandomService {
    type Error = Infallible;
    fn try_next_u32(&mut self) -> core::result::Result<u32, Self::Error> {
        let mut data = [0; 4];
        self.generate_random_bytes(Buffer::from_mut_array(&mut data))
            .expect("Generating rand bytes should never fail");
        Ok(u32::from_ne_bytes(data))
    }

    fn try_next_u64(&mut self) -> core::result::Result<u64, Self::Error> {
        let mut data = [0; 8];
        self.generate_random_bytes(Buffer::from_mut_array(&mut data))
            .expect("Generating rand bytes should never fail");
        Ok(u64::from_ne_bytes(data))
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> core::result::Result<(), Self::Error> {
        self.generate_random_bytes(Buffer::from_mut_array(dst))
            .expect("Generating rand bytes should never fail");
        Ok(())
    }
}

// Global RNG source
static G_RNG: Mutex<Option<spl::SplCsrngGenerator>> = Mutex::new(None);

pub fn initialize() -> Result<()> {
    let mut guard = G_RNG.lock();
    if guard.is_none() {
        *guard = Some(spl::SplCsrngGenerator::new()?);
    }

    Ok(())
}

pub fn finalize() {
    *G_RNG.lock() = None;
}

#[inline]
pub fn get_rng() -> Result<spl::SplCsrngGenerator> {
    G_RNG
        .lock()
        .clone()
        .ok_or(nx::rc::ResultNotInitialized::make())
}

mod spl {

    use core::convert::Infallible;

    use rand::{TryCryptoRng, TryRng};

    use super::*;

    /// Represents a pseudo-RNG using [`spl`][`crate::service::spl`]'s [`RandomService`] interface
    #[derive(Clone)]
    pub struct SplCsrngGenerator {
        csrng: Arc<RandomService>,
    }

    impl SplCsrngGenerator {
        /// Creates a new [`SplCsrngGenerator`]
        pub fn new() -> Result<Self> {
            Ok(Self {
                csrng: Arc::new(service::new_service_object::<RandomService>()?),
            })
        }
    }

    impl TryRng for SplCsrngGenerator {
        type Error = Infallible;
        fn try_next_u32(&mut self) -> core::result::Result<u32, Self::Error> {
            let mut data = [0; 4];
            self.csrng
                .generate_random_bytes(Buffer::from_mut_array(&mut data))
                .expect("Generating rand bytes should never fail");
            Ok(u32::from_ne_bytes(data))
        }
        fn try_next_u64(&mut self) -> core::result::Result<u64, Self::Error> {
            let mut data = [0; 8];
            self.csrng
                .generate_random_bytes(Buffer::from_mut_array(&mut data))
                .expect("Generating rand bytes should never fail");
            Ok(u64::from_ne_bytes(data))
        }
        fn try_fill_bytes(&mut self, dst: &mut [u8]) -> core::result::Result<(), Self::Error> {
            self.csrng
                .generate_random_bytes(Buffer::from_mut_array(dst))
                .expect("Generating rand bytes should never fail");
            Ok(())
        }
    }

    impl TryCryptoRng for SplCsrngGenerator {}
}
