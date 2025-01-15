//! Memory (heap) support and utils

extern crate alloc as core_alloc;
use ::alloc::sync::Arc;
use core::marker::Unsize;
use core::ops::CoerceUnsized;
use core::ops::Deref;
use crate::result::ResultBase;
use crate::svc;
use crate::sync::Mutex;

pub mod alloc;

/// Flushes data cache at a certain memory region
/// 
/// # Arguments
/// 
/// * `address`: Memory region address
/// * `size`: Memory region size
/// # Safety
///
/// Null pointers are OK as we are just doing cache invalidation, not accessing the pointer.
#[inline(always)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn flush_data_cache(address: *mut u8, size: usize) {
    extern "C" {
        fn __nx_mem_flush_data_cache(address: *mut u8, size: usize);
    }

    unsafe {
        __nx_mem_flush_data_cache(address, size);
    }
}

/// Blocks thread until the memory region specified has the permission passed
/// 
/// # Arguments
/// 
/// * `address`: The address to query for memory permissions
/// * `permissions`: The memory permission to wait on
/// 
/// Note that if multiple permissions are specified (e.g. `MemoryPermission::Read | MemoryPermission::Write`), the function will return if *any* specified permission is present.
#[inline(always)]
pub fn wait_for_permission(address: svc::Address, permission: svc::MemoryPermission, timeout: Option<usize>) -> crate::result::Result<()> {
    let mut iteration: usize = 0;
    loop {
        let (memory, _) = svc::query_memory(address)?;
        if memory.permission.contains(permission) {
            return Ok(());
        }
        if let Some(timeout) = timeout && timeout <= (100_000 * iteration) {
            // The timeout has been set and has already expired
            return Err(svc::rc::ResultTimedOut::make());
        }
        iteration += 1;
        let _ = crate::thread::sleep(100_000);
    }
}


#[repr(transparent)]
pub struct Shared<T: ?Sized> {
    pub(crate) inner: Arc<Mutex<T>>
}
impl<T> Shared<T> {
    pub fn new(val: T) -> Self {
        Self { inner: Arc::new(Mutex::new(val)) }
    }
}
impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<T: ?Sized> Deref for Shared<T> {
    type Target = Mutex<T>;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Shared<U>> for Shared<T> {}
