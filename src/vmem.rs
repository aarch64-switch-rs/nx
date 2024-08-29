//! Virtual memory support

use core::ptr;
use crate::result::*;
use crate::sync;
use crate::svc;
use crate::mem::alloc;

/// Represents a virtual region of memory
#[derive(Copy, Clone, Default)]
pub struct VirtualRegion {
    /// The start address of the region
    pub start: usize,
    /// The end address of the region
    pub end: usize
}

impl VirtualRegion {
    /// Creates an empty [`VirtualRegion`] with invalid address values
    #[inline]
    pub const fn null() -> Self {
        Self { start: 0, end: 0 }
    }

    /// Returns whether the specified address is contained in the region
    ///
    /// # Arguments
    ///
    /// * `address` - The address to check
    #[inline]
    pub const fn contains(&self, address: usize) -> bool {
        (address >= self.start) && (address < self.end)
    }
}

pub enum VirtualRegionType {
    Stack,
    Heap,
    LegacyAlias
}

static mut G_STACK_REGION: VirtualRegion = VirtualRegion::null();
static mut G_HEAP_REGION: VirtualRegion = VirtualRegion::null();
static mut G_LEGACY_ALIAS_REGION: VirtualRegion = VirtualRegion::null();
static mut G_ADDRESS_SPACE: VirtualRegion = VirtualRegion::null();
static mut G_CURRENT_ADDRESS: usize = 0;
static mut G_LOCK: sync::Mutex = sync::Mutex::new();

/// Gets the current process's address space [`VirtualRegion`]
/// 
/// Note that [`initialize()`] must have been called before for the region to be valid (although it's automatically called on [`rrt0`][`crate::rrt0`])
pub fn get_address_space() -> VirtualRegion {
    unsafe {
        let _ = sync::ScopedLock::new(&mut *ptr::addr_of_mut!(G_LOCK));
        G_ADDRESS_SPACE
    }
}

/// Gets the current process's stack [`VirtualRegion`]
/// 
/// Note that [`initialize()`] must have been called before for the region to be valid (although it's automatically called on [`rrt0`][`crate::rrt0`])
pub fn get_stack_region() -> VirtualRegion {
    unsafe {
        let _ = sync::ScopedLock::new(&mut *ptr::addr_of_mut!(G_LOCK));
        G_STACK_REGION
    }
}

/// Gets the current process's heap [`VirtualRegion`]
/// 
/// Note that [`initialize()`] must have been called before for the region to be valid (although it's automatically called on [`rrt0`][`crate::rrt0`])
pub fn get_heap_region() -> VirtualRegion {
    unsafe {
        let _ = sync::ScopedLock::new(&mut *ptr::addr_of_mut!(G_LOCK));
        G_HEAP_REGION
    }
}

/// Gets the current process's legacy alias [`VirtualRegion`]
/// 
/// Note that [`initialize()`] must have been called before for the region to be valid (although it's automatically called on [`rrt0`][`crate::rrt0`])
pub fn get_legacy_alias_region() -> VirtualRegion {
    unsafe {
        let _ = sync::ScopedLock::new(&mut *ptr::addr_of_mut!(G_LOCK));
        G_LEGACY_ALIAS_REGION
    }
}

fn read_region_info(region: *mut VirtualRegion, address_info_id: svc::InfoId, size_info_id: svc::InfoId) -> Result<()> {
    let address = svc::get_info(address_info_id, svc::CURRENT_PROCESS_PSEUDO_HANDLE, 0)? as usize;
    let size = svc::get_info(size_info_id, svc::CURRENT_PROCESS_PSEUDO_HANDLE, 0)? as usize;

    unsafe {
        // Safety: pointer will never be null here
        (*region).start = address;
        (*region).end = address + size;
    }
    Ok(())
}

/// Initializes virtual memory support
/// 
/// This internally retrieves all the current process's memory [`VirtualRegion`]s
/// 
/// This is automatically called on [`rrt0`][`crate::rrt0`]
pub fn initialize() -> Result<()> {
    unsafe {
        let _ = sync::ScopedLock::new(&mut *ptr::addr_of_mut!(G_LOCK));
        read_region_info(ptr::addr_of_mut!(G_ADDRESS_SPACE), svc::InfoId::AslrRegionAddress, svc::InfoId::AslrRegionSize)?;
        read_region_info(ptr::addr_of_mut!(G_STACK_REGION), svc::InfoId::StackRegionAddress, svc::InfoId::StackRegionSize)?;
        read_region_info(ptr::addr_of_mut!(G_HEAP_REGION), svc::InfoId::HeapRegionAddress, svc::InfoId::HeapRegionSize)?;
        read_region_info(ptr::addr_of_mut!(G_LEGACY_ALIAS_REGION), svc::InfoId::AliasRegionAddress, svc::InfoId::AliasRegionSize)?;
    }
    Ok(())
}

/// Finds available virtual memory for the specified size, returning it's address
///
/// Note that [`initialize()`] must have been called before for this to succeed (although it's automatically called on [`rrt0`][`crate::rrt0`])
/// 
/// # Arguments
/// 
/// * `size`: The size of the virtual memory to allocate
pub fn allocate(size: usize) -> Result<*mut u8> {
    unsafe {
        let _ = sync::ScopedLock::new(&mut *ptr::addr_of_mut!(G_LOCK));

        let mut address = G_CURRENT_ADDRESS;

        loop {
            address += alloc::PAGE_ALIGNMENT;

            if !G_ADDRESS_SPACE.contains(address) {
                address = G_ADDRESS_SPACE.start;
            }

            let current_address = address + size;
            let (memory_info, _) = svc::query_memory(address as *mut u8)?;
            let info_address = memory_info.base_address + memory_info.size;
            if memory_info.state != svc::MemoryState::Free {
                address = info_address;
                continue;
            }

            if current_address > info_address {
                address = info_address;
                continue;
            }

            let end = current_address - 1;

            if G_STACK_REGION.contains(address) || G_STACK_REGION.contains(end) {
                address = G_STACK_REGION.end;
                continue;
            }
            if G_HEAP_REGION.contains(address) || G_HEAP_REGION.contains(end) {
                address = G_HEAP_REGION.end;
                continue;
            }
            if G_LEGACY_ALIAS_REGION.contains(address) || G_LEGACY_ALIAS_REGION.contains(end) {
                address = G_LEGACY_ALIAS_REGION.end;
                continue;
            }

            break;
        }

        G_CURRENT_ADDRESS = address + size;
        Ok(address as *mut u8)
    }
}