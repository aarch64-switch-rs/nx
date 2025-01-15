//! Allocator implementation and definitions

use crate::result::*;
use crate::util::PointerAndSize;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr;
use core::ptr::NonNull;

extern crate alloc;
use alloc::alloc::Allocator;

use alloc::alloc::Global;
pub use alloc::alloc::Layout;

pub const PAGE_ALIGNMENT: usize = 0x1000;

pub mod rc;

use alloc::alloc::AllocError;

impl From<AllocError> for ResultCode {
    fn from(_value: AllocError) -> Self {
        ResultCode::new(rc::ResultOutOfMemory::get_value())
    }
}

// TODO: be able to change the global allocator?

/// Represents a heap allocator for this library
/// # Safety
///
/// As with the regular Allocator trait, the `delete` function can only be called on pointers produced by the same implementation's `new`
pub unsafe trait AllocatorEx: Allocator {
    /// Allocates a new heap value
    #[allow(clippy::new_ret_no_self)]
    #[allow(clippy::wrong_self_convention)]
    fn new<T>(&self) -> Result<NonNull<T>> {
        let layout = Layout::new::<T>();
        match self.allocate(layout) {
            Ok(allocation) => Ok(allocation.cast()),
            Err(_) => rc::ResultOutOfMemory::make_err()
        }
    }

    /// Releases a heap value
    ///
    /// The value must have been created using [`AllocatorEx::new`]
    ///
    /// # Arguments
    ///
    /// * `t`: Heap value address
    fn delete<T>(&self, t: *mut T) {
        let layout = Layout::new::<T>();
        unsafe { self.deallocate(NonNull::new_unchecked(t.cast()), layout) };
    }
}

unsafe impl AllocatorEx for Global {}

#[global_allocator]
static GLOBAL_ALLOCATOR: linked_list_allocator::LockedHeap =
    linked_list_allocator::LockedHeap::empty();

/// Initializes the global allocator with the given address and size.
/// Returns a bool to indicate if the memory was consumed by the allocator
///
/// # Arguments
///
/// * `heap`: The heap address and size
pub fn initialize(heap: PointerAndSize) -> bool {
    unsafe {
        GLOBAL_ALLOCATOR
            .lock()
            .init(heap.address, heap.size)
    };
    false
}


/// Gets whether heap allocations are enabled
///
/// The library may internally disable them in exceptional cases: for instance, to avoid exception handlers to allocate heap memory if the exception cause is actually an OOM situation
pub fn is_enabled() -> bool {
    let alloc_guard = GLOBAL_ALLOCATOR.lock();
    alloc_guard.size() != 0
}

/// Represents a wrapped and manually managed heap value
///
/// Note that a [`Buffer`] is able to hold both a single value or an array of values of the provided type
pub struct Buffer<T, A: Allocator = Global> {
    /// The actual heap value
    pub ptr: *mut T,
    /// The memory's layout
    pub layout: Layout,
    /// The allocator used to request the buffer
    allocator: A,
}

impl<T> Buffer<T> {
    /// Creates a new, invalid [`Buffer`]
    #[inline]
    pub const fn empty() -> Self {
        Self {
            ptr: ptr::null_mut(),
            layout: Layout::new::<u8>(), // Dummy value
            allocator: Global,
        }
    }

    /// Gets whether this [`Buffer`] is valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.ptr.is_null()
    }

    /// Creates a new [`Buffer`] using the global allocator
    ///
    /// # Arguments
    ///
    /// * `align`: The align to use
    /// * `count`: The count of values to allocate
    pub fn new(align: usize, count: usize) -> Result<Self> {
        let layout = Layout::from_size_align(count * mem::size_of::<T>(), align)
            .map_err(|_| ResultCode::new(rc::ResultLayoutError::get_value()))?;
        let allocator = Global;
        let ptr = allocator.allocate(layout)?.as_ptr().cast();
        Ok(Self {
            ptr,
            layout,
            allocator,
        })
    }

    pub fn into_raw(value: Self) -> *mut [T] {
        let no_drop = ManuallyDrop::new(value);
        core::ptr::slice_from_raw_parts_mut(no_drop.ptr, no_drop.layout.size() / mem::size_of::<T>())
    }

    /// Releases the [`Buffer`]
    ///
    /// The [`Buffer`] becomes invalid after this
    pub fn release(&mut self) {
        if self.is_valid() {
            unsafe {
                self.allocator.deallocate(NonNull::new_unchecked(self.ptr.cast()), self.layout);
            }
            self.ptr = core::ptr::null_mut();
        }
        
    }
}

impl<T, A: Allocator> Buffer<T, A> {
    /// Creates a new [`Buffer`] using a given allocator
    ///
    /// # Arguments
    ///
    /// * `align`: The align to use
    /// * `count`: The count of values to allocate
    /// * `allocator`: The allocator to use
    pub fn new_in(align: usize, count: usize, allocator: A) -> Result<Self> {
        let layout = Layout::from_size_align(count * mem::size_of::<T>(), align)
            .map_err(|_| ResultCode::new(rc::ResultLayoutError::get_value()))?;
        let ptr = allocator.allocate(layout)?.as_ptr().cast();
        Ok(Self {
            ptr,
            layout,
            allocator,
        })
    }
}

impl<T, A: Allocator> Drop for Buffer<T, A> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                self.allocator
                    .deallocate(NonNull::new_unchecked(self.ptr.cast()), self.layout);
            }
        }
    }
}
