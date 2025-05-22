//! Memory (heap) support and utils

extern crate alloc as core_alloc;
use crate::result::ResultBase;
use crate::svc;

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
    unsafe extern "C" {
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
pub fn wait_for_permission(
    address: svc::Address,
    permission: svc::MemoryPermission,
    timeout: Option<usize>,
) -> crate::result::Result<()> {
    let mut iteration: usize = 0;
    loop {
        let (memory, _) = svc::query_memory(address)?;
        if memory.permission.contains(permission) {
            return Ok(());
        }
        if timeout.is_some() && timeout <= Some(100_000 * iteration) {
            // The timeout has been set and has already expired
            return Err(svc::rc::ResultTimedOut::make());
        }
        iteration += 1;
        let _ = crate::thread::sleep(100_000);
    }
}
