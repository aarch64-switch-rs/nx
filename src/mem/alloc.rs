//! Allocator implementation and definitions

use crate::result::*;
use crate::util::PointerAndSize;
use crate::sync;
use core::ptr;
use core::mem;
use core::ptr::NonNull;
use core::result::Result as CoreResult;

extern crate alloc;
use alloc::alloc::Allocator;

use alloc::alloc::Global;
use alloc::alloc::GlobalAlloc;
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
pub unsafe trait AllocatorEx: Allocator {
    /// Allocates a new heap value
    fn new<T>(&mut self) -> Result<*mut T> {
        let layout = Layout::new::<T>();
        match self.allocate(layout) {
            Ok(allocation) => Ok(allocation.as_ptr().cast()),
            Err(_) => rc::ResultOutOfMemory::make_err()
        }
    }

    /// Releases a heap value
    /// 
    /// The value must have been created using [`Allocator::new`]
    /// 
    /// # Arguments
    /// 
    /// * `t`: Heap value address
    fn delete<T>(&mut self, t: *mut T) {
        let layout = Layout::new::<T>();
        unsafe {self.deallocate(NonNull::new_unchecked(t.cast()), layout)};
    }
}

unsafe impl AllocatorEx for Global{}

extern crate linked_list_allocator;
use linked_list_allocator::Heap as LinkedListAllocator;

unsafe impl<A: Allocator> GlobalAlloc for sync::Locked<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.get().allocate(layout).unwrap().as_ptr().as_mut_ptr()
    }

    unsafe fn dealloc(&self, addr: *mut u8, layout: Layout) {
        self.get().deallocate(ptr::NonNull::new_unchecked(addr), layout);
    }
}

struct LateInitAllocator {
    initialized: bool,
    inner: LinkedListAllocator
}

impl LateInitAllocator {
    const fn new() -> Self {
        Self { initialized: false, inner: LinkedListAllocator::empty() }
    }

    unsafe fn init(&mut self, bottom: *mut u8, size: usize) {
        assert!(!self.initialized, "Heap already initialized");
        self.inner.init(bottom, size);
        self.initialized = true;
    }
}

unsafe impl Allocator for sync::Locked<LateInitAllocator> {
    fn allocate(&self, layout: Layout) -> CoreResult<NonNull<[u8]>, AllocError> {
        let handle = self.get();
        // if compiled in debug mode, the allocator will panic when allocating without initialising the allocator
        debug_assert!(handle.initialized, "Allocator not initialized");
        // if compiled in release mode, the allocator will return an OOM error as there is no memory available for the allocator
        if !handle.initialized {return Err(AllocError);}
        match handle.inner.allocate_first_fit(layout) {
            Ok(non_null_addr) => Ok(NonNull::slice_from_raw_parts(non_null_addr, layout.size())),
            Err(_) => Err(AllocError)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.get().inner.deallocate(ptr, layout);
    }

}

unsafe impl GlobalAlloc for sync::Locked<LateInitAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocate(layout).unwrap().as_ptr().as_mut_ptr()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocate(NonNull::new_unchecked(ptr), layout)
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: sync::Locked<LateInitAllocator> = sync::Locked::new(false, LateInitAllocator::new());

/// Initializes the global allocator with the given address and size.
/// Returns a bool to indicate if the memory was consumed by the allocator
/// 
/// # Arguments
/// 
/// * `heap`: The heap address and size
pub fn initialize(heap: PointerAndSize) -> bool {
    unsafe {
        let handle = GLOBAL_ALLOCATOR.get();
        if !handle.initialized {
            handle.inner.init(heap.address, heap.size);
            handle.initialized = true;
            return true;
        } else {
            return false;
        }
    }
}

/*
pub(crate) fn set_enabled(enabled: bool) {
    unsafe {
        G_ALLOCATOR_ENABLED = enabled;
    }
}
*/

/// Gets whether heap allocations are enabled
/// 
/// The library may internally disable them in exceptional cases: for instance, to avoid exception handlers to allocate heap memory if the exception cause is actually an OOM situation
pub fn is_enabled() -> bool {
        GLOBAL_ALLOCATOR.get().initialized
}

/*
/// Allocates heap memory using the global allocator
/// 
/// # Arguments
/// 
/// * `align`: The memory alignment
/// * `size`: The memory size
pub fn allocate(align: usize, size: usize) -> Result<*mut u8> {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, align);
        match GLOBAL_ALLOCATOR.get().allocate(layout){
            Ok(p) => p.as_ptr().as_mut_ptr(),
            err(e)
        }.map(|p| p.as_ptr().as_mut_ptr()).ma
    }
}

/// Releases allocated memory
/// 
/// # Arguments
/// 
/// * `addr`: The memory address
/// * `align`: The memory alignment
/// * `size`: The memory size
pub fn release(addr: *mut u8, align: usize, size: usize) {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, align);
        GLOBAL_ALLOCATOR.get().release(addr, layout);
    }
}

/// Creates a new heap value using the global allocator
pub fn new<T>() -> Result<*mut T> {
    unsafe {
        GLOBAL_ALLOCATOR.get().new::<T>()
    }
}

/// Deletes a heap value
/// 
/// The value must have been created using the global allocator
/// 
/// # Arguments
/// 
/// * `t`: The heap value
pub fn delete<T>(t: *mut T) {
    unsafe {
        GLOBAL_ALLOCATOR.get().delete(t);
    }
}
*/
/// Represents a wrapped and manually managed heap value
/// 
/// Note that a [`Buffer`] is able to hold both a single value or an array of values of the provided type
pub struct Buffer<T, A: Allocator = Global> {
    /// The actual heap value
    pub ptr: *mut T,
    /// The memory's layout
    pub layout: Layout,
    /// The allocator used to request the buffer
    allocator: A
}

impl<T> Buffer<T> {
    /// Creates a new, invalid [`Buffer`]
    #[inline]
    pub const fn empty() -> Self {
        Self {
            ptr: ptr::null_mut(),
            layout: Layout::new::<u8>(), // Dummy value
            allocator: Global
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
        let layout = Layout::from_size_align(count * mem::size_of::<T>(), align).map_err(|_| ResultCode::new(rc::ResultLayoutError::get_value()))?;
        let allocator = Global;
        let ptr = allocator.allocate(layout)?.as_ptr().cast();
        Ok(Self {
            ptr,
            layout,
            allocator
        })
    }

    pub fn into_raw(self) -> *mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.ptr, self.layout.size() / mem::size_of::<T>()) as *mut [T]
        }
    }

    /// Releases the [`Buffer`]
    /// 
    /// The [`Buffer`] becomes invalid after this
    pub fn release(&mut self) {
        unsafe {self.allocator.deallocate(NonNull::new_unchecked(self.ptr.cast()), self.layout);}
        *self = Self::empty();
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
        let layout = Layout::from_size_align(count * mem::size_of::<T>(), align).map_err(|_| ResultCode::new(rc::ResultLayoutError::get_value()))?;
        let ptr = allocator.allocate(layout)?.as_ptr().cast();
        Ok(Self {
            ptr,
            layout,
            allocator
        })
    }
}

impl<T, A: Allocator> Drop for Buffer<T, A> {
    fn drop(&mut self) {
        unsafe {self.allocator.deallocate(NonNull::new_unchecked(self.ptr.cast()), self.layout);}
    }
}